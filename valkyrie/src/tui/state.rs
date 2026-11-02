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
    collections::HashMap,
    sync::{Mutex, MutexGuard, PoisonError},
    time::{Duration, Instant},
};

use valkyrie_chess::{ChessBoard, FEN, Piece, Side, Square};

use crate::tui::{arrow::Arrow, canvas::Rect, editor::Editor, graphics::Protocol};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placed {
    pub piece: usize,
    pub white: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub squares: [[Option<Placed>; 8]; 8],
    pub fen: String,
}

impl Position {
    pub fn new(board: &ChessBoard) -> Self {
        let mut squares = [[None; 8]; 8];

        for (row, rank) in (0..8u8).rev().enumerate() {
            for file in 0..8u8 {
                let square = Square::from_coords(rank, file);
                let piece = board.piece_on_square(square);

                if piece != Piece::NONE {
                    squares[row][usize::from(file)] = Some(Placed {
                        piece: usize::from(piece),
                        white: board.color_on_square(square) == Side::WHITE,
                    });
                }
            }
        }

        Self {
            squares,
            fen: standard_castling(&FEN::from(board).to_string()),
        }
    }

    pub fn black_to_move(&self) -> bool {
        self.field(1) == "b"
    }

    pub fn field(&self, index: usize) -> &str {
        self.fen.split_whitespace().nth(index).unwrap_or("-")
    }
}

fn move_arrow(text: &str) -> Option<Arrow> {
    let bytes = text.as_bytes();
    let square = |file: u8, rank: u8| {
        let row = b'8'.checked_sub(rank).filter(|row| *row < 8)?;
        let file = file.checked_sub(b'a').filter(|file| *file < 8)?;

        Some((usize::from(row), usize::from(file)))
    };

    Some(Arrow::new(
        square(*bytes.first()?, *bytes.get(1)?)?,
        square(*bytes.get(2)?, *bytes.get(3)?)?,
    ))
}

fn standard_castling(fen: &str) -> String {
    let mut fields: Vec<String> = fen.split_whitespace().map(str::to_string).collect();

    if let Some(castling) = fields.get_mut(2) {
        *castling = castling
            .chars()
            .map(|symbol| match symbol {
                'H' => 'K',
                'A' => 'Q',
                'h' => 'k',
                'a' => 'q',
                other => other,
            })
            .collect();
    }

    fields.join(" ")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Pending,
    Info,
    Result,
    Error,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub tone: Tone,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Setting(String),
    Copy { label: &'static str, text: String },
    Suggestion(usize),
    Quit,
}

#[derive(Debug, Clone)]
pub struct Region {
    pub area: Rect,
    pub target: Target,
}

#[derive(Debug, Clone)]
pub struct Edit {
    pub name: String,
    pub value: String,
    pub fresh: bool,
}

#[derive(Debug, Default)]
pub struct State {
    pub options: HashMap<String, String>,
    pub response: Option<Response>,
    pub busy: bool,
    pub editor: Editor,
    pub editing: Option<Edit>,
    pub hover: Option<(usize, usize)>,
    pub regions: Vec<Region>,
    pub help_open: bool,
    pub position: Option<Position>,
    pub screen: Screen,
    pub perft: Option<PerftRun>,
    pub intro: Option<Instant>,
    pub placed: Option<Instant>,
    pub stopped_at: Option<Instant>,
    pub copied: Option<Copied>,
    pub graphics: Option<Protocol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    #[default]
    Welcome,
    Perft,
}

#[derive(Debug, Clone, Copy)]
pub struct PerftOutcome {
    pub nodes: u64,
    pub elapsed: Duration,
    pub stopped: bool,
}

#[derive(Debug, Clone)]
pub struct PerftRun {
    pub depth: u8,
    pub bulk: bool,
    pub moves: usize,
    pub queue: Vec<Arrow>,
    pub splits: Vec<(String, u64)>,
    pub started: Instant,
    pub outcome: Option<PerftOutcome>,
}

impl PerftRun {
    pub fn new(depth: u8, bulk: bool, board: &ChessBoard) -> Self {
        let mut queue = Vec::new();

        board.map_legal_moves(|mv| queue.extend(move_arrow(&mv.to_string(false))));

        Self {
            depth,
            bulk,
            moves: queue.len(),
            queue,
            splits: Vec::new(),
            started: Instant::now(),
            outcome: None,
        }
    }

    pub fn done(&self) -> bool {
        self.outcome.is_some()
    }

    pub fn current(&self) -> Option<Arrow> {
        self.queue
            .get(self.splits.len())
            .copied()
            .filter(|_| !self.done())
    }

    pub fn nodes(&self) -> u64 {
        self.outcome.map_or_else(
            || self.splits.iter().map(|(_, nodes)| nodes).sum(),
            |outcome| outcome.nodes,
        )
    }

    pub fn elapsed(&self) -> Duration {
        self.outcome
            .map_or_else(|| self.started.elapsed(), |outcome| outcome.elapsed)
    }
}

impl State {
    pub fn place(&mut self, position: Position) {
        if self.position.as_ref() != Some(&position) {
            self.placed = Some(Instant::now());
        }

        self.position = Some(position);
    }

    pub fn respond(&mut self, tone: Tone, text: &str) {
        let Some(line) = text.lines().map(str::trim).find(|line| !line.is_empty()) else {
            return;
        };

        self.response = Some(Response {
            tone,
            text: line.to_string(),
        });
    }

    pub fn option_value<'a>(&'a self, name: &str, default: &'a str) -> &'a str {
        self.options
            .get(&name.to_lowercase())
            .map_or(default, String::as_str)
    }

    pub fn hovers(&self, area: Rect) -> bool {
        self.hover.is_some_and(|position| area.contains(position))
    }

    pub fn hovered_target(&self) -> Option<&Target> {
        let position = self.hover?;

        self.regions
            .iter()
            .rev()
            .find(|region| region.area.contains(position))
            .map(|region| &region.target)
    }
}

pub fn lock(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Debug, Clone)]
pub struct Copied {
    pub label: String,
    pub message: String,
    pub at: Instant,
}
