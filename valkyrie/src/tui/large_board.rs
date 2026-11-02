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

use crate::tui::{
    arrow::Arrow,
    canvas::{Canvas, Rect},
    palette::{BACKGROUND, DARK_SQUARE, LIGHT_SQUARE, MUTED, Rgb},
    state::Position,
    theme::Style,
    widgets,
};

pub const SQUARE_WIDTH: usize = 6;
pub const SQUARE_HEIGHT: usize = 3;
pub const LABEL_WIDTH: usize = 2;
pub const LARGE_BOARD_WIDTH: usize = LABEL_WIDTH + SQUARE_WIDTH * 8;
pub const LARGE_BOARD_ROWS: usize = SQUARE_HEIGHT * 8 + 1;

const FILE_OFFSET: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PieceImage {
    pub piece: usize,
    pub white: bool,
    pub opacity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SquareImage {
    pub x: usize,
    pub y: usize,
    pub columns: usize,
    pub rows: usize,
    pub shown: (usize, usize),
    pub square: Rgb,
    pub board: Rect,
    pub piece: Option<PieceImage>,
    pub arrow: Option<Arrow>,
}

impl SquareImage {
    pub fn same_place(&self, other: &Self) -> bool {
        (self.x, self.y, self.board) == (other.x, other.y, other.board)
    }

    pub const fn plain(self) -> Self {
        Self {
            piece: None,
            arrow: None,
            ..self
        }
    }
}

pub fn images(
    board: Rect,
    label: usize,
    (columns, rows): (usize, usize),
    position: &Position,
    opacity: impl Fn(usize) -> u8,
    arrow: Option<Arrow>,
) -> Vec<SquareImage> {
    let flipped = position.black_to_move();
    let orient = |index: usize| if flipped { 7 - index } else { index };
    let arrow = arrow.map(|arrow| if flipped { arrow.flipped() } else { arrow });
    let order = widgets::placement_order(position);
    let mut images = Vec::new();

    for (shown_row, slots) in order.iter().enumerate() {
        for (shown_file, slot) in slots.iter().enumerate() {
            let (row, file) = (orient(shown_row), orient(shown_file));
            let piece = position.squares[row][file]
                .map(|piece| PieceImage {
                    piece: piece.piece,
                    white: piece.white,
                    opacity: opacity(*slot),
                })
                .filter(|piece| piece.opacity > 0);
            let arrow = arrow.filter(|arrow| arrow.touches((shown_row, shown_file)));

            if piece.is_some() || arrow.is_some() {
                images.push(SquareImage {
                    x: board.x + label + shown_file * columns,
                    y: board.y + shown_row * rows,
                    columns,
                    rows,
                    shown: (shown_row, shown_file),
                    square: square_color(row, file),
                    board,
                    piece,
                    arrow,
                });
            }
        }
    }

    images
}

pub fn square_color(row: usize, file: usize) -> Rgb {
    if (row + file).is_multiple_of(2) {
        LIGHT_SQUARE
    } else {
        DARK_SQUARE
    }
}

pub fn draw(canvas: &mut Canvas, x: usize, y: usize, position: &Position, fade: f32) {
    let label = Style::new(BACKGROUND.lerp(MUTED, fade));
    let flipped = position.black_to_move();
    let orient = |index: usize| if flipped { 7 - index } else { index };

    for shown_row in 0..8 {
        for shown_file in 0..8 {
            let color = BACKGROUND.lerp(square_color(orient(shown_row), orient(shown_file)), fade);

            canvas.fill(
                Rect::new(
                    x + LABEL_WIDTH + shown_file * SQUARE_WIDTH,
                    y + shown_row * SQUARE_HEIGHT,
                    SQUARE_WIDTH,
                    SQUARE_HEIGHT,
                ),
                Style::new(color).on(color),
            );
        }

        canvas.put(
            x,
            y + shown_row * SQUARE_HEIGHT + SQUARE_HEIGHT / 2,
            &(8 - orient(shown_row)).to_string(),
            label,
        );
        canvas.set(
            x + LABEL_WIDTH + shown_row * SQUARE_WIDTH + FILE_OFFSET,
            y + SQUARE_HEIGHT * 8,
            char::from(b'a' + orient(shown_row) as u8),
            label,
        );
    }
}
