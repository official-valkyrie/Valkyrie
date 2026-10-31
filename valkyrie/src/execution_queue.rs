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

use std::{cmp::Reverse, collections::BinaryHeap};

pub mod stage {
    pub const CONFIGURE: u16 = 100;
    pub const SETUP: u16 = 200;
    pub const RUN: u16 = 300;
}

#[derive(Debug, Default)]
pub struct ExecutionQueue {
    entries: BinaryHeap<Reverse<(u16, usize, String)>>,
    supplied: usize,
}

impl ExecutionQueue {
    pub fn push(&mut self, stage: u16, line: String) {
        self.entries.push(Reverse((stage, self.supplied, line)));
        self.supplied += 1;
    }
}

impl Iterator for ExecutionQueue {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        self.entries.pop().map(|Reverse((_, _, line))| line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain(lines: &[(u16, &str)]) -> Vec<String> {
        let mut queue = ExecutionQueue::default();

        for &(stage, line) in lines {
            queue.push(stage, line.to_string());
        }

        queue.collect()
    }

    #[test]
    fn earlier_stages_run_first() {
        assert_eq!(
            drain(&[
                (stage::RUN, "bench"),
                (stage::SETUP, "position startpos"),
                (stage::CONFIGURE, "setoption name Hash value 64"),
            ]),
            ["setoption name Hash value 64", "position startpos", "bench"]
        );
    }

    #[test]
    fn same_stage_keeps_the_supplied_order() {
        assert_eq!(
            drain(&[
                (stage::RUN, "perft 3"),
                (stage::CONFIGURE, "setoption name Hash value 64"),
                (stage::RUN, "draw"),
                (stage::RUN, "bench"),
            ]),
            ["setoption name Hash value 64", "perft 3", "draw", "bench"]
        );
    }
}
