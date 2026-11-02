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
    sync::{Arc, Mutex, mpsc::Sender},
    time::{Duration, Instant},
};

use valkyrie_chess::{ChessBoard, FEN};
use valkyrie_engine::prelude::*;

use crate::tui::{
    completion::Catalog,
    events::Events,
    graphics::Protocol,
    perft,
    renderer::Renderer,
    state::{PerftOutcome, PerftRun, Position, Screen, State, Tone, lock},
};

#[derive(Debug)]
struct Running {
    renderer: Renderer,
    events: Events,
}

#[derive(Debug)]
pub struct TerminalView {
    state: Arc<Mutex<State>>,
    catalog: Arc<Catalog>,
    sender: Sender<String>,
    running: Mutex<Option<Running>>,
}

impl TerminalView {
    pub fn new(sender: Sender<String>, catalog: Catalog) -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                position: Some(Position::new(&ChessBoard::from(&FEN::start_position()))),
                graphics: Protocol::detect(),
                ..State::default()
            })),
            catalog: Arc::new(catalog),
            sender,
            running: Mutex::new(None),
        }
    }

    fn update(&self, change: impl FnOnce(&mut State)) {
        change(&mut lock(&self.state));
    }

    fn finish_perft(&self, nodes: u64, elapsed: Duration, stopped: bool) {
        self.update(|state| {
            state.busy = false;

            let Some(run) = &mut state.perft else {
                return;
            };

            run.outcome = Some(PerftOutcome {
                nodes,
                elapsed,
                stopped,
            });

            let summary = format!("Perft {}: {} nodes", run.depth, perft::group(nodes));

            if stopped {
                state.respond(Tone::Error, &format!("{summary} before it was stopped"));
            } else {
                state.respond(Tone::Result, &summary);
            }
        })
    }

    fn respond(&self, tone: Tone, text: &str) {
        self.update(|state| state.respond(tone, text))
    }
}

impl View for TerminalView {
    fn start(&self) {
        let mut running = self
            .running
            .lock()
            .unwrap_or_else(|error| error.into_inner());

        if running.is_none() {
            *running = Some(Running {
                renderer: Renderer::spawn(Arc::clone(&self.state), Arc::clone(&self.catalog)),
                events: Events::spawn(
                    Arc::clone(&self.state),
                    Arc::clone(&self.catalog),
                    self.sender.clone(),
                ),
            });
        }
    }

    fn finish(&self) {
        let running = self
            .running
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take();

        if let Some(mut running) = running {
            running.events.stop();
            running.renderer.stop();
        }
    }

    fn message(&self, level: MessageLevel, text: &str) {
        self.respond(
            match level {
                MessageLevel::Info => Tone::Info,
                MessageLevel::Error => Tone::Error,
            },
            text,
        )
    }

    fn clear(&self) {
        self.update(|state| state.response = None)
    }

    fn uci_ok(&self) {
        self.respond(Tone::Info, "uciok")
    }

    fn ready_ok(&self) {
        self.respond(Tone::Info, "readyok")
    }

    fn board(&self, board: &ChessBoard) {
        self.update(|state| state.position = Some(Position::new(board)))
    }

    fn position_changed(&self, board: &ChessBoard) {
        self.update(|state| {
            state.place(Position::new(board));
            state.respond(Tone::Info, "Position set");
        })
    }

    fn new_game(&self, board: &ChessBoard) {
        self.update(|state| {
            state.place(Position::new(board));
            state.respond(Tone::Info, "New game");
        })
    }

    fn option_changed(&self, name: &str, value: &str) {
        self.update(|state| {
            state.options.insert(name.to_lowercase(), value.to_string());
            state.respond(Tone::Info, &format!("{name} set to {value}"));
        })
    }

    fn button_pressed(&self, name: &str) {
        self.respond(Tone::Info, &format!("{name} triggered"))
    }

    fn perft_started(&self, board: &ChessBoard, depth: u8, bulk: bool) {
        self.update(|state| {
            state.position = Some(Position::new(board));
            state.perft = Some(PerftRun::new(depth, bulk, board));
            state.screen = Screen::Perft;
            state.intro = Some(Instant::now());
            state.busy = true;
            state.editing = None;
        })
    }

    fn perft_move(&self, mv: Move, nodes: u64, chess960: bool) {
        self.update(|state| {
            if let Some(run) = &mut state.perft {
                run.splits.push((mv.to_string(chess960), nodes));
            }
        })
    }

    fn perft_finished(&self, nodes: u64, elapsed: Duration) {
        self.finish_perft(nodes, elapsed, false)
    }

    fn perft_stopped(&self, nodes: u64, elapsed: Duration) {
        self.finish_perft(nodes, elapsed, true)
    }

    fn bench_finished(&self, nodes: u64, elapsed: Duration) {
        let milliseconds = elapsed.as_millis().max(1);

        self.respond(
            Tone::Result,
            &format!(
                "Bench {nodes} nodes · {} nps",
                u128::from(nodes) * 1000 / milliseconds
            ),
        )
    }

    fn search_started(&self, _limits: &SearchLimits) {
        self.update(|state| state.busy = true)
    }

    fn search_ended(&self, _report: &SearchReport) {
        self.update(|state| state.busy = false)
    }

    fn best_move(&self, mv: Option<Move>, chess960: bool) {
        let text = mv.map_or_else(|| String::from("0000"), |mv| mv.to_string(chess960));

        self.update(|state| {
            state.busy = false;
            state.respond(Tone::Result, &format!("Best move {text}"));
        })
    }
}
