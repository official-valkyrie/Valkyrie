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

#[derive(Debug, Clone, Default)]
pub struct SearchLimits {
    nodes: Option<u64>,
    depth: Option<u64>,
    infinite: bool,
    time: Option<u64>,
    increment: Option<u64>,
    moves_to_go: Option<u64>,
    move_time: Option<u64>,
}

impl SearchLimits {
    pub const fn nodes(&self) -> Option<u64> {
        self.nodes
    }
    pub const fn depth(&self) -> Option<u64> {
        self.depth
    }
    pub const fn is_infinite(&self) -> bool {
        self.infinite
    }
    pub const fn time(&self) -> Option<u64> {
        self.time
    }
    pub const fn increment(&self) -> Option<u64> {
        self.increment
    }
    pub const fn moves_to_go(&self) -> Option<u64> {
        self.moves_to_go
    }
    pub const fn move_time(&self) -> Option<u64> {
        self.move_time
    }
    pub fn set_nodes(&mut self, nodes: Option<u64>) {
        self.nodes = nodes;
    }
    pub fn set_depth(&mut self, depth: Option<u64>) {
        self.depth = depth;
    }
    pub fn set_infinite(&mut self, infinite: bool) {
        self.infinite = infinite;
    }
    pub fn set_time(&mut self, time: Option<u64>) {
        self.time = time;
    }
    pub fn set_increment(&mut self, increment: Option<u64>) {
        self.increment = increment;
    }
    pub fn set_moves_to_go(&mut self, moves_to_go: Option<u64>) {
        self.moves_to_go = moves_to_go;
    }
    pub fn set_move_time(&mut self, move_time: Option<u64>) {
        self.move_time = move_time;
    }
}
