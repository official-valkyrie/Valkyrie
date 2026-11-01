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

use crate::{Bitboard, Square};

pub struct Rays;
impl Rays {
    #[inline]
    pub fn get_ray(from: Square, to: Square) -> Bitboard {
        RAYS[usize::from(from)][usize::from(to)]
    }
}

static RAYS: [[Bitboard; 64]; 64] = {
    let mut result = [[Bitboard::EMPTY; 64]; 64];
    let mut from_square_index = 0;
    while from_square_index < 64 {
        let mut to_square_index = 0;
        while to_square_index < 64 {
            let from_square = Square::from_value(from_square_index);
            let to_square = Square::from_value(to_square_index);
            result[from_square_index as usize][to_square_index as usize] =
                generate_ray(from_square, to_square);
            to_square_index += 1;
        }
        from_square_index += 1;
    }

    result
};

const fn generate_ray(from: Square, to: Square) -> Bitboard {
    let rank_increment = (to.get_rank() as i32 - from.get_rank() as i32).signum();
    let file_increment = (to.file() as i32 - from.file() as i32).signum();

    if rank_increment == 0 && file_increment == 0 {
        return Bitboard::EMPTY;
    }

    let mut result = 0u64;
    let mut rank = from.get_rank() as i32 + rank_increment;
    let mut file = from.file() as i32 + file_increment;

    while rank >= 0 && rank <= 7 && file >= 0 && file <= 7 {
        let current_square = Square::from_coords(rank as u8, file as u8);
        result |= Bitboard::from_square(current_square).get_value();
        if to.equals(current_square) {
            return Bitboard::from_value(result);
        }
        rank += rank_increment;
        file += file_increment;
    }

    Bitboard::EMPTY
}
