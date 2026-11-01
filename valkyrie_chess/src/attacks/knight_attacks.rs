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

pub struct KnightAttacks;
impl KnightAttacks {
    pub const ATTACK_TABLE: [Bitboard; 64] = {
        let mut result = [Bitboard::EMPTY; 64];
        let mut square_index = 0usize;
        while square_index < 64 {
            let bb = Bitboard::from_square(Square::from_value(square_index as u8));
            let mut attack_map: u64 = 0;
            if Bitboard::FILE_A
                .inverse()
                .and(bb.shift_left(17))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(17).get_value()
            }
            if Bitboard::FILE_H
                .inverse()
                .and(bb.shift_left(15))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(15).get_value()
            }
            if Bitboard::FILE_A
                .or(Bitboard::FILE_B)
                .inverse()
                .and(bb.shift_left(10))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(10).get_value()
            }
            if Bitboard::FILE_H
                .or(Bitboard::FILE_G)
                .inverse()
                .and(bb.shift_left(6))
                .is_not_empty()
            {
                attack_map |= bb.shift_left(6).get_value()
            }
            if Bitboard::FILE_H
                .inverse()
                .and(bb.shift_right(17))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(17).get_value()
            }
            if Bitboard::FILE_A
                .inverse()
                .and(bb.shift_right(15))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(15).get_value()
            }
            if Bitboard::FILE_H
                .or(Bitboard::FILE_G)
                .inverse()
                .and(bb.shift_right(10))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(10).get_value()
            }
            if Bitboard::FILE_A
                .or(Bitboard::FILE_B)
                .inverse()
                .and(bb.shift_right(6))
                .is_not_empty()
            {
                attack_map |= bb.shift_right(6).get_value()
            }
            result[square_index] = Bitboard::from_value(attack_map);
            square_index += 1;
        }
        result
    };
}
