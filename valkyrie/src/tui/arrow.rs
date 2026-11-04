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

use crate::tui::palette::{GOLD_LIGHT, Rgb};

const DEFAULT_WIDTH: f32 = 0.16;
const DEFAULT_OPACITY: f32 = 0.8;
const HEAD_LENGTH: f32 = 2.8;
const HEAD_WIDTH: f32 = 4.0;
const TIP_INSET: f32 = 0.1;
const SQUARE_REACH: f32 = 0.71;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arrow {
    pub from: (usize, usize),
    pub to: (usize, usize),
    pub color: Rgb,
    pub width: f32,
    pub opacity: f32,
}

impl Arrow {
    pub const fn new(from: (usize, usize), to: (usize, usize)) -> Self {
        Self {
            from,
            to,
            color: GOLD_LIGHT,
            width: DEFAULT_WIDTH,
            opacity: DEFAULT_OPACITY,
        }
    }

    pub const fn color(self, color: Rgb) -> Self {
        Self { color, ..self }
    }

    pub const fn width(self, width: f32) -> Self {
        Self { width, ..self }
    }

    pub const fn opacity(self, opacity: f32) -> Self {
        Self { opacity, ..self }
    }

    pub fn flipped(self) -> Self {
        let flip = |(row, file): (usize, usize)| (7 - row, 7 - file);

        Self {
            from: flip(self.from),
            to: flip(self.to),
            ..self
        }
    }

    pub fn touches(&self, (row, file): (usize, usize)) -> bool {
        self.distance((file as f32 + 0.5, row as f32 + 0.5)) < SQUARE_REACH
    }

    pub fn alpha(&self, point: (f32, f32), pixels_per_square: f32) -> f32 {
        let coverage = (0.5 - self.distance(point) * pixels_per_square).clamp(0.0, 1.0);

        coverage * self.opacity
    }

    fn distance(&self, (x, y): (f32, f32)) -> f32 {
        let center = |(row, file): (usize, usize)| (file as f32 + 0.5, row as f32 + 0.5);
        let (start, end) = (center(self.from), center(self.to));
        let (dx, dy) = (end.0 - start.0, end.1 - start.1);
        let length = dx.hypot(dy).max(f32::EPSILON);
        let (ux, uy) = (dx / length, dy / length);
        let (px, py) = (x - start.0, y - start.1);
        let along = px * ux + py * uy;
        let across = (px * uy - py * ux).abs();

        let tip = length - TIP_INSET;
        let head_length = self.width * HEAD_LENGTH;
        let head_half = self.width * HEAD_WIDTH / 2.0;
        let base = (tip - head_length).max(0.0);

        let shaft = (along - along.clamp(0.0, base)).hypot(across) - self.width / 2.0;
        let edge =
            ((along - tip) * head_half + across * head_length) / head_half.hypot(head_length);
        let head = (base - along).max(edge);

        shaft.min(head)
    }
}
