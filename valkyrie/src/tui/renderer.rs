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
    panic,
    sync::{
        Arc, Mutex, Once,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

use crate::{
    terminal,
    tui::{
        animation,
        canvas::Canvas,
        completion::{Catalog, suggest},
        palette::BACKGROUND,
        screens::{self, Frame},
        state::{State, lock},
    },
};

pub const FPS: u32 = 30;

const FRAME_TIME: Duration = Duration::from_nanos(1_000_000_000 / FPS as u64);
const FALLBACK_SIZE: (u16, u16) = (80, 24);
const ENTER: &str = "\x1b[?1049h\x1b[?25l\x1b[?7l\x1b[?2004h";
const LEAVE: &str = "\x1b]111\x1b\\\x1b[0m\x1b[?2004l\x1b[?7h\x1b[?25h\x1b[?1049l";
const FRAME_BEGIN: &str = "\x1b[?2026h";
const FRAME_END: &str = "\x1b[?2026l";
const CLEAR: &str = "\x1b[0m\x1b[2J";
const PARK: &str = "\x1b[1;1H";
const RESIZE_SETTLE: Duration = Duration::from_millis(500);
const FULL_REPAINT: Duration = Duration::from_secs(2);

static ACTIVE: AtomicBool = AtomicBool::new(false);
static PANIC_HOOK: Once = Once::new();

pub fn restore() {
    if ACTIVE.swap(false, Ordering::SeqCst) {
        let _ = crossterm::execute!(std::io::stdout(), DisableMouseCapture);
        terminal::write_unlocked(LEAVE);
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

fn enter() {
    let _ = crossterm::terminal::enable_raw_mode();
    ACTIVE.store(true, Ordering::SeqCst);
    terminal::write_unlocked(&format!(
        "{ENTER}\x1b]11;rgb:{:02x}/{:02x}/{:02x}\x1b\\",
        BACKGROUND.0, BACKGROUND.1, BACKGROUND.2
    ));
    let _ = crossterm::execute!(std::io::stdout(), EnableMouseCapture);
}

fn install_panic_hook() {
    PANIC_HOOK.call_once(|| {
        let previous = panic::take_hook();

        panic::set_hook(Box::new(move |info| {
            restore();
            previous(info);
        }));
    });
}

#[derive(Debug)]
pub struct Renderer {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Renderer {
    pub fn spawn(state: Arc<Mutex<State>>, catalog: Arc<Catalog>) -> Self {
        install_panic_hook();
        enter();

        let stop = Arc::new(AtomicBool::new(false));
        let handle = {
            let stop = Arc::clone(&stop);
            thread::spawn(move || run(&state, &catalog, &stop))
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

impl Drop for Renderer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run(state: &Mutex<State>, catalog: &Catalog, stop: &AtomicBool) {
    let mut canvas = Canvas::new(0, 0);
    let mut shown = Vec::new();
    let mut settling_until = Instant::now();
    let mut last_full = Instant::now();
    let opened = Instant::now();

    while !stop.load(Ordering::SeqCst) && ACTIVE.load(Ordering::SeqCst) {
        let started = Instant::now();

        let (width, height) = terminal::size().unwrap_or(FALLBACK_SIZE);
        let resized = canvas.resize(usize::from(width), usize::from(height));

        if resized {
            settling_until = started + RESIZE_SETTLE;
        }

        let full = resized || started < settling_until || started - last_full >= FULL_REPAINT;

        if full {
            shown.clear();
            last_full = started;
        }

        canvas.clear();
        let elapsed = opened.elapsed().as_secs_f32() / animation::DURATION.as_secs_f32();

        compose(
            &mut canvas,
            state,
            catalog,
            (elapsed < 1.0).then_some(elapsed),
        );

        let changes = canvas.diff(&mut shown);
        let clear = if resized { CLEAR } else { "" };

        if !changes.is_empty() && ACTIVE.load(Ordering::SeqCst) {
            terminal::write_unlocked(&format!("{FRAME_BEGIN}{clear}{changes}{PARK}{FRAME_END}"));
        }

        thread::sleep(FRAME_TIME.saturating_sub(started.elapsed()));
    }

    restore();
}

fn compose(canvas: &mut Canvas, state: &Mutex<State>, catalog: &Catalog, progress: Option<f32>) {
    let mut state = lock(state);
    let suggestions = suggest(catalog, &state.editor.before_cursor());

    let regions = screens::draw(
        canvas,
        &Frame {
            state: &state,
            catalog,
            suggestions: &suggestions,
            progress,
        },
    );

    state.regions = regions;
}
