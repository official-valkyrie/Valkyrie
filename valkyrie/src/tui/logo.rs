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

use std::sync::OnceLock;

use crate::tui::{
    animation,
    canvas::Canvas,
    palette::{BACKGROUND, Rgb, STEEL},
    theme::Style,
};

const ART: &str = include_str!("logo.txt");
const COLORS: &str = include_str!("logo_colors.txt");

const ORIGIN_ROW_SHARE: f32 = 0.62;
const CELL_ASPECT: f32 = 2.0;
const SPREAD: f32 = 0.55;
const JITTER: f32 = 0.08;
const SPAN: f32 = 0.35;
const SETTLE: f32 = 0.55;
const LETTERS: [char; 8] = ['V', 'A', 'L', 'K', 'Y', 'R', 'I', 'E'];

pub struct Logo {
    pub width: usize,
    pub height: usize,
    rows: Vec<Vec<Option<(char, Rgb)>>>,
}

impl Logo {
    pub fn get() -> &'static Logo {
        static LOGO: OnceLock<Logo> = OnceLock::new();

        LOGO.get_or_init(Logo::build)
    }

    fn build() -> Self {
        let rows: Vec<Vec<Option<(char, Rgb)>>> = ART
            .lines()
            .zip(COLORS.lines())
            .map(|(symbols, colors)| {
                symbols
                    .chars()
                    .zip(colors.split(' '))
                    .map(|(symbol, color)| {
                        (symbol != ' ').then(|| (symbol, Rgb::parse(color).unwrap_or(STEEL)))
                    })
                    .collect()
            })
            .collect();

        Self {
            width: rows.iter().map(Vec::len).max().unwrap_or(0),
            height: rows.len(),
            rows,
        }
    }

    pub fn draw(&self, canvas: &mut Canvas, x: usize, y: usize, progress: Option<f32>) {
        let origin = (
            (self.width as f32 - 1.0) / 2.0,
            self.height as f32 * ORIGIN_ROW_SHARE,
        );
        let distance = |column: usize, row: usize| {
            ((column as f32 - origin.0) / CELL_ASPECT).hypot(row as f32 - origin.1)
        };
        let reach = distance(0, 0).max(distance(0, self.height));

        for (row, cells) in self.rows.iter().enumerate() {
            for (column, (symbol, color)) in cells
                .iter()
                .enumerate()
                .filter_map(|(column, cell)| Some((column, (*cell)?)))
            {
                let Some(progress) = progress else {
                    canvas.set(x + column, y + row, symbol, Style::new(color));
                    continue;
                };

                let delay = SPREAD * distance(column, row) / reach + JITTER * noise(column, row);
                let amount = animation::smooth((progress - delay) / SPAN);

                if amount <= 0.0 {
                    continue;
                }

                let symbol = if amount < SETTLE {
                    LETTERS[(column * 7 + row * 13 + (progress * 40.0) as usize) % LETTERS.len()]
                } else {
                    symbol
                };

                canvas.set(
                    x + column,
                    y + row,
                    symbol,
                    Style::new(BACKGROUND.lerp(color, amount)),
                );
            }
        }
    }
}

fn noise(column: usize, row: usize) -> f32 {
    let hash = (column as u32).wrapping_mul(73_856_093) ^ (row as u32).wrapping_mul(19_349_663);

    (hash % 1000) as f32 / 1000.0
}
