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

use std::time::Duration;

use valkyrie_chess::{ChessBoard, Move};

use crate::engine::{Engine, OptionSpec, SearchContext, builder::EngineConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLevel {
    Info,
    Error,
}

pub trait Display<C: EngineConfig>: std::fmt::Debug + Send + Sync {
    fn message(&self, _level: MessageLevel, _text: &str) {}

    fn identify(&self, _name: &str, _author: &str) {}

    fn option(&self, _spec: &OptionSpec) {}

    fn handshake_done(&self) {}

    fn ready(&self) {}

    fn board(&self, _board: &ChessBoard) {}

    fn perft_started(&self, _board: &ChessBoard, _depth: u8, _bulk: bool) {}

    fn perft_move(&self, _mv: Move, _nodes: u64, _engine: &Engine<C>) {}

    fn perft_finished(&self, _nodes: u64, _elapsed: Duration) {}

    fn bench_finished(&self, _nodes: u64, _elapsed: Duration) {}

    fn search_started(&self, _context: &SearchContext<C>, _engine: &Engine<C>) {}

    fn search_update(&self, _context: &SearchContext<C>, _engine: &Engine<C>) {}

    fn search_ended(&self, _context: &SearchContext<C>, _engine: &Engine<C>) {}

    fn best_move(&self, _mv: Move, _engine: &Engine<C>) {}
}

#[derive(Debug)]
pub struct NoDisplay;

impl<C: EngineConfig> Display<C> for NoDisplay {}
