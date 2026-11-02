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

use crate::tui::canvas::{Canvas, Rect};

pub const DURATION: Duration = Duration::from_millis(400);

const ROW_DELAY: f32 = 0.05;
const ROW_SPREAD: f32 = 0.55;
const ROW_SPAN: f32 = 0.35;
const PIECES_END: f32 = 0.95;
const BOARD_DELAY: f32 = 0.5;

pub fn smooth(amount: f32) -> f32 {
    let amount = amount.clamp(0.0, 1.0);

    amount * amount * (3.0 - 2.0 * amount)
}

pub fn row_start(y: usize, height: usize) -> f32 {
    ROW_DELAY + ROW_SPREAD * y as f32 / height.max(1) as f32
}

pub fn board_start(y: usize, height: usize) -> f32 {
    row_start(y, height) + ROW_SPAN * BOARD_DELAY
}

pub fn pieces_placed(progress: f32, start: f32, total: usize) -> usize {
    if progress < start {
        return 0;
    }

    let placed = (progress - start) / (PIECES_END - start).max(f32::EPSILON);

    ((placed * total as f32).floor() as usize).min(total)
}

pub fn reveal_rows(canvas: &mut Canvas, progress: f32, keep: &[Rect]) {
    let height = canvas.height();

    for y in 0..height {
        let amount = smooth((progress - row_start(y, height)) / ROW_SPAN);

        if amount >= 1.0 {
            continue;
        }

        for x in 0..canvas.width() {
            if !keep.iter().any(|area| area.contains((x, y))) {
                canvas.fade(x, y, amount);
            }
        }
    }
}
