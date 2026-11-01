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

use crate::prelude::*;

const EVAL_SCALE: f32 = 400.0;
const SCORE_EPSILON: f32 = 0.0001;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct WinChance(f32);

impl WinChance {
    #[inline]
    pub fn new(probability: f32) -> Self {
        debug_assert!(
            (0.0..=1.0).contains(&probability),
            "win chance {probability} is outside [0, 1]"
        );
        Self(probability)
    }

    #[inline]
    pub fn from_cp(cp: f32) -> Self {
        Self(1.0 / (1.0 + (-cp / EVAL_SCALE).exp()))
    }
}

impl Payload for WinChance {
    const WIN: Self = Self(1.0);
    const DRAW: Self = Self(0.5);
    const LOSS: Self = Self(0.0);

    #[inline]
    fn as_scalar(&self) -> f32 {
        self.0
    }

    #[inline]
    fn flip(&mut self) {
        self.0 = 1.0 - self.0;
    }
}

impl Score for WinChance {
    #[inline]
    fn cp(&self) -> i32 {
        let probability = self.0.clamp(SCORE_EPSILON, 1.0 - SCORE_EPSILON);
        (-EVAL_SCALE * (1.0 / probability - 1.0).ln()) as i32
    }
}
