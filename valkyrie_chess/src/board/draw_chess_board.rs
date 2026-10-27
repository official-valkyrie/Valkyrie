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

use crate::{
    FEN,
    Piece,
    Side,
    Square,
    board::ChessBoard,
};

const PIECE_SYMBOLS: [char; 6] = ['p', 'n', 'b', 'r', 'q', 'k'];

impl ChessBoard {
    pub fn draw_board(&self) {
        let mut result = String::new();

        for rank in (0..8).rev() {
            result.push(char::from(b'1' + rank));

            for file in 0..8 {
                let square = Square::from_coords(rank, file);
                let piece = self.piece_on_square(square);

                let symbol = if piece == Piece::NONE {
                    '.'
                } else if self.color_on_square(square) == Side::WHITE {
                    PIECE_SYMBOLS[usize::from(piece)].to_ascii_uppercase()
                } else {
                    PIECE_SYMBOLS[usize::from(piece)]
                };

                result.push(' ');
                result.push(symbol);
            }

            result.push('\n');
        }

        println!("{result}  a b c d e f g h");
        println!("FEN: {}", FEN::from(self));
    }
}
