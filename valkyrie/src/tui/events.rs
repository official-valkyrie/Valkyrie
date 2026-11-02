/*
    This file is part of Valkyrie.

    Copyright (C) 2026 Tomasz Jaworski and Valkyrie Contributors

    Valkyrie is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Valkyrie is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Valkyrie.  If not, see <https://gnu.org>.

    Additional terms under GNU GPL version 3 section 7

    1. Linking Exception:
    If you modify this Program, or any covered work, by linking or
    combining it with the ONNX Runtime and/or NVIDIA TensorRT libraries
    (or a modified version of those libraries), containing parts covered
    by the terms of the respective ONNX Runtime and NVIDIA license
    agreements, the licensors of this Program grant you additional
    permission to convey the resulting work.

   2. Attribution Requirement (Section 7b):
    Under Section 7(b) of the GNU General Public License, version 3,
    the licensors of this Program require that any modified or redistributed
    versions of this software must prominently preserve original author
    attributions and credit to the Valkyrie Contributors in the software
    documentation.
*/

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};
use valkyrie_engine::prelude::OptionKind;

use crate::{
    interrupt, terminal,
    tui::{
        completion::Catalog,
        editor::Outcome,
        state::{Copied, Edit, PerftRun, Screen, State, Target, Tone, lock},
    },
};

const POLL_INTERVAL: Duration = Duration::from_millis(40);
const STOP_GRACE: Duration = Duration::from_millis(500);
const STOP_COMMAND: &str = "stop";

#[derive(Debug)]
pub struct Events {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Events {
    pub fn spawn(state: Arc<Mutex<State>>, catalog: Arc<Catalog>, sender: Sender<String>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let handle = {
            let stop = Arc::clone(&stop);
            thread::spawn(move || run(&state, &catalog, &sender, &stop))
        };

        Self {
            stop,
            handle: Some(handle),
        }
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);

        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Events {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run(state: &Mutex<State>, catalog: &Catalog, sender: &Sender<String>, stop: &AtomicBool) {
    while !stop.load(Ordering::SeqCst) {
        match event::poll(POLL_INTERVAL) {
            Ok(true) => {}
            Ok(false) => continue,
            Err(_) => break,
        }

        let Ok(event) = event::read() else {
            break;
        };

        match event {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                key_pressed(state, catalog, sender, key)
            }
            Event::Mouse(mouse) => mouse_used(state, catalog, sender, mouse),
            Event::Paste(text) => {
                let mut state = lock(state);

                match state.editing.as_mut() {
                    Some(edit) => type_into(edit, &text),
                    None => state.editor.insert(&text),
                }
            }
            _ => {}
        }
    }
}

fn submit(state: &mut State, sender: &Sender<String>, line: String) {
    state.respond(Tone::Pending, &line);
    let _ = sender.send(line);
}

fn key_pressed(state: &Mutex<State>, catalog: &Catalog, sender: &Sender<String>, key: KeyEvent) {
    let control = key.modifiers.contains(KeyModifiers::CONTROL);

    if control && matches!(key.code, KeyCode::Char('c' | 'C')) {
        ctrl_c(state, sender);
        return;
    }

    let mut state = lock(state);

    if state.help_open {
        state.help_open = false;
        return;
    }

    if state.screen == Screen::Perft {
        if !state.perft.as_ref().is_some_and(PerftRun::done) {
            return;
        }

        state.screen = Screen::Welcome;
        state.intro = Some(Instant::now());
    }

    if let Some(edit) = state.editing.as_mut() {
        match key.code {
            KeyCode::Enter => {
                if let Some(edit) = state.editing.take() {
                    let line = format!("setoption name {} value {}", edit.name, edit.value.trim());
                    submit(&mut state, sender, line);
                }
            }
            KeyCode::Esc => state.editing = None,
            KeyCode::Backspace => {
                if edit.fresh {
                    edit.value.clear();
                } else {
                    edit.value.pop();
                }
                edit.fresh = false;
            }
            KeyCode::Char(symbol) if !control => type_into(edit, &symbol.to_string()),
            _ => {}
        }
        return;
    }

    match state.editor.handle(key, catalog) {
        Outcome::Nothing => {}
        Outcome::Interrupt => {
            drop(state);
            interrupt::ctrl_c();
        }
        Outcome::Submit(line)
            if catalog
                .command(&line)
                .is_some_and(|command| command.name == crate::tui::HELP_COMMAND) =>
        {
            state.respond(Tone::Pending, &line);
            state.help_open = true;
        }
        Outcome::Submit(line) => submit(&mut state, sender, line),
    }
}

fn type_into(edit: &mut Edit, text: &str) {
    if edit.fresh {
        edit.value.clear();
        edit.fresh = false;
    }

    edit.value
        .extend(text.chars().filter(|symbol| !symbol.is_control()));
}

fn mouse_used(state: &Mutex<State>, catalog: &Catalog, sender: &Sender<String>, mouse: MouseEvent) {
    let mut state = lock(state);
    let position = (usize::from(mouse.column), usize::from(mouse.row));

    state.hover = Some(position);

    if state.help_open && matches!(mouse.kind, MouseEventKind::Down(_)) {
        state.help_open = false;
        return;
    }

    if mouse.kind != MouseEventKind::Down(MouseButton::Left) {
        return;
    }

    match state.hovered_target().cloned() {
        Some(Target::Setting(name)) => activate(&mut state, catalog, sender, &name),
        Some(Target::Suggestion(index)) => state.editor.choose(index, catalog),
        Some(Target::Copy { label, text }) => copy(&mut state, label, &text),
        Some(Target::Quit) => {
            state.editing = None;
            let command = if state.busy { "stop" } else { "quit" };
            submit(&mut state, sender, command.to_string());
        }
        None => state.editing = None,
    }
}

fn activate(state: &mut State, catalog: &Catalog, sender: &Sender<String>, name: &str) {
    let Some(spec) = catalog.option(name) else {
        return;
    };

    let current = state.option_value(&spec.name, &spec.default).to_string();

    let next = match spec.kind {
        OptionKind::Check => Some(if current.eq_ignore_ascii_case("true") {
            "false".to_string()
        } else {
            "true".to_string()
        }),
        OptionKind::Combo => {
            let position = spec
                .variants
                .iter()
                .position(|variant| variant.eq_ignore_ascii_case(&current));
            let next = position.map_or(0, |index| (index + 1) % spec.variants.len().max(1));

            spec.variants.get(next).map(|variant| variant.to_string())
        }
        OptionKind::Spin | OptionKind::String => {
            state.editing = Some(Edit {
                name: spec.name.clone(),
                value: current,
                fresh: true,
            });
            None
        }
        OptionKind::Button => {
            submit(state, sender, format!("setoption name {}", spec.name));
            None
        }
    };

    if let Some(value) = next {
        state.editing = None;
        submit(
            state,
            sender,
            format!("setoption name {} value {value}", spec.name),
        );
    }
}

fn copy(state: &mut State, original: &str, text: &str) {
    if terminal::copy_to_clipboard(text) {
        let mut letters = original.chars();
        let label: String = letters
            .next()
            .map(|first| first.to_uppercase().chain(letters).collect())
            .unwrap_or_default();

        state.copied = Some(Copied {
            label: original.to_string(),
            message: format!("{label} copied to clipboard"),
            at: Instant::now(),
        });
    } else {
        state.respond(Tone::Error, "Could not access the clipboard");
    }
}

fn ctrl_c(state: &Mutex<State>, sender: &Sender<String>) {
    let mut state = lock(state);

    if state.busy {
        state.stopped_at = Some(Instant::now());
        let _ = sender.send(STOP_COMMAND.to_string());
        return;
    }

    if state
        .stopped_at
        .is_some_and(|stopped| stopped.elapsed() < STOP_GRACE)
    {
        return;
    }

    drop(state);
    interrupt::ctrl_c();
}
