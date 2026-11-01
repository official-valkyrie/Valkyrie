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

use crate::base_structures::ZobristKey;

const HISTORY_CAPACITY: usize = crate::HALF_MOVE_LIMIT as usize + 1;

#[derive(Debug, Clone, Copy)]
pub struct MoveHistory([ZobristKey; HISTORY_CAPACITY], usize);
impl MoveHistory {
    pub fn new() -> Self {
        Self([ZobristKey::default(); HISTORY_CAPACITY], 0)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.1
    }

    #[inline]
    pub fn hash(&self) -> u128 {
        let mut result = 0u128;
        for value in 0..self.1 {
            let hash = (u64::from(self.0[value]) as u128) << 64 | u64::from(self.0[value]) as u128;
            result ^= (hash >> value) << 7;
        }

        result &= !0b1111111;
        result |= self.1 as u128 & 0b1111111;

        result &= !((u64::MAX as u128) << 64);
        result |= (u64::from(self.0[self.1 - 1]) as u128) << 64;

        result
    }

    #[inline]
    pub fn push(&mut self, key: ZobristKey) {
        self.0[self.1] = key;
        self.1 += 1;
    }

    #[inline]
    pub fn reset(&mut self) {
        self.1 = 0;
    }

    #[inline]
    pub fn get_repetitions(&self, key: ZobristKey) -> i32 {
        let mut repetitions = 0;
        for value in 0..self.1 {
            if key != self.0[value] {
                continue;
            }

            repetitions += 1;
        }
        repetitions
    }
}

impl Default for MoveHistory {
    fn default() -> Self {
        Self::new()
    }
}
