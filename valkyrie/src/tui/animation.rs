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

use std::time::{Duration, Instant};

use crate::tui::{
    canvas::{Canvas, Rect},
    state::Screen,
};

pub const PIECE_STEPS: u8 = 6;

const WELCOME_DURATION: Duration = Duration::from_millis(400);
const REPORT_DURATION: Duration = Duration::from_millis(1000);
const BOARD_DURATION: Duration = Duration::from_millis(1000);

const ROW_DELAY: f32 = 0.05;
const ROW_SPREAD: f32 = 0.55;
const ROW_SPAN: f32 = 0.35;
const BOARD_FADE: f32 = 0.1;
const PIECE_FADE: f32 = 0.3;
const PIECE_DELAY: f32 = 0.014;
const SPINNER: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
const SPINNER_FRAME: Duration = Duration::from_millis(80);
const SWEEP_STEP: Duration = Duration::from_millis(25);

#[derive(Debug, Clone, Copy)]
pub struct BoardClock {
    elapsed: f32,
    start: f32,
}

impl BoardClock {
    pub fn new(screen: Screen, progress: f32, y: usize, height: usize) -> Self {
        let length = duration(screen).as_secs_f32();

        Self {
            elapsed: progress * length,
            start: row_start(y, height) * length,
        }
    }

    pub fn placing(since: Instant) -> Option<Self> {
        let elapsed = since.elapsed();

        (elapsed < BOARD_DURATION).then(|| Self {
            elapsed: elapsed.as_secs_f32(),
            start: -BOARD_FADE,
        })
    }

    pub fn board(self) -> f32 {
        smooth((self.elapsed - self.start) / BOARD_FADE)
    }

    pub fn piece(self, slot: usize) -> f32 {
        let begin = self.start + BOARD_FADE + PIECE_DELAY * slot as f32;

        smooth((self.elapsed - begin) / PIECE_FADE)
    }
}

pub fn spinner(since: Instant) -> char {
    let frame = since.elapsed().as_millis() / SPINNER_FRAME.as_millis();

    SPINNER[frame as usize % SPINNER.len()]
}

pub fn sweep(since: Instant, width: usize, length: usize) -> std::ops::Range<usize> {
    let travel = width.saturating_sub(length).max(1);
    let step = (since.elapsed().as_millis() / SWEEP_STEP.as_millis()) as usize % (travel * 2);
    let start = if step < travel {
        step
    } else {
        travel * 2 - step
    };

    start..(start + length).min(width)
}

pub fn smooth(amount: f32) -> f32 {
    let amount = amount.clamp(0.0, 1.0);

    amount * amount * (3.0 - 2.0 * amount)
}

fn row_start(y: usize, height: usize) -> f32 {
    ROW_DELAY + ROW_SPREAD * y as f32 / height.max(1) as f32
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

pub const fn duration(screen: Screen) -> Duration {
    match screen {
        Screen::Welcome => WELCOME_DURATION,
        Screen::Perft | Screen::Search => REPORT_DURATION,
    }
}

pub fn length(screen: Screen) -> Duration {
    duration(screen).max(BOARD_DURATION)
}
