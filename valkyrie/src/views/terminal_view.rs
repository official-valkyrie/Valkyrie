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
        atomic::{AtomicU64, Ordering},
        mpsc::Sender,
    },
    time::{Duration, Instant},
};

use valkyrie_chess::{ChessBoard, FEN};
use valkyrie_engine::prelude::*;

use crate::tui::{
    completion::Catalog,
    events::Events,
    graphics::Protocol,
    renderer::{FRAME_TIME, Renderer},
    report, search,
    state::{Applying, PerftOutcome, PerftRun, Position, Screen, SearchRun, State, Tone, lock},
};

const REPORT_CHECK_INTERVAL: u64 = 16;
const SLOW_OPTION: Duration = Duration::from_millis(250);

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
    last_report: AtomicU64,
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
            last_report: AtomicU64::new(0),
        }
    }

    fn update(&self, change: impl FnOnce(&mut State)) {
        let mut state = lock(&self.state);

        change(&mut state);
        self.release_queue(&mut state);
    }

    fn release_queue(&self, state: &mut State) {
        while state.applying.is_none() {
            let Some(line) = state.queue.pop_front() else {
                break;
            };

            state.respond(Tone::Pending, &line);
            state.applying = Applying::parse(&line);
            let _ = self.sender.send(line);
        }
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

            let summary = format!("Perft {}: {} nodes", run.depth, report::group(nodes));

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

            let finished = state
                .applying
                .take()
                .filter(|applying| applying.name.eq_ignore_ascii_case(name));

            match finished {
                Some(applying) if applying.since.elapsed() >= SLOW_OPTION => state.respond(
                    Tone::Info,
                    &format!(
                        "{name} set to {} in {}",
                        applying.shown_value(),
                        report::time(applying.since.elapsed())
                    ),
                ),
                Some(applying) => state.respond(
                    Tone::Info,
                    &format!("{name} set to {}", applying.shown_value()),
                ),
                None => state.respond(Tone::Info, &format!("{name} set to {value}")),
            }
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
        self.last_report.store(0, Ordering::Relaxed);
        self.update(|state| {
            state.search = Some(SearchRun::default());
            state.screen = Screen::Search;
            state.intro = Some(Instant::now());
            state.busy = true;
            state.editing = None;
        })
    }

    fn wants_report(&self, stats: &SearchStats) -> bool {
        if !stats.iterations().is_multiple_of(REPORT_CHECK_INTERVAL) {
            return false;
        }

        let now = stats.elapsed_ms();
        let due = now >= self.last_report.load(Ordering::Relaxed) + FRAME_TIME.as_millis() as u64;

        if due {
            self.last_report.store(now, Ordering::Relaxed);
        }

        due
    }

    fn report(&self, report: &SearchReport) {
        self.update(|state| state.update_search(report))
    }

    fn search_ended(&self, report: &SearchReport) {
        self.update(|state| state.update_search(report))
    }

    fn best_move(&self, mv: Option<Move>, chess960: bool) {
        self.update(|state| {
            state.busy = false;

            let Some(run) = &mut state.search else {
                let text = mv.map_or_else(|| String::from("0000"), |mv| mv.to_string(chess960));
                state.respond(Tone::Result, &format!("Best move {text}"));
                return;
            };

            let best = mv.map_or_else(|| String::from("none"), |mv| run.best_san(mv));
            let score = run
                .lines
                .first()
                .map_or_else(|| "-".to_string(), |line| search::score_text(line.score));
            let summary = format!(
                "Best move {best} · {score} · depth {} · max depth {}",
                run.depth, run.seldepth
            );

            run.best = Some(best);
            state.respond(Tone::Result, &summary);
        })
    }
}
