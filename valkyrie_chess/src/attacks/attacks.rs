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
    Bitboard, Square,
    attacks::{KingAttacks, KnightAttacks, PawnsAttacks, slider_attacks},
    base_structures::Side,
};

pub struct Attacks;
impl Attacks {
    #[inline(always)]
    pub const fn get_king_attacks(square: Square) -> Bitboard {
        KingAttacks::ATTACK_TABLE[square.get_value() as usize]
    }

    #[inline(always)]
    pub const fn get_knight_attacks(square: Square) -> Bitboard {
        KnightAttacks::ATTACK_TABLE[square.get_value() as usize]
    }

    #[inline(always)]
    pub const fn get_pawn_attacks(square: Square, attacker_side: Side) -> Bitboard {
        PawnsAttacks::ATTACK_TABLE[attacker_side.get_value() as usize][square.get_value() as usize]
    }

    #[inline(always)]
    pub fn get_bishop_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        unsafe {
            let params = &slider_attacks::LOOKUP[64 + square.get_value() as usize];
            let idx = slider_attacks::calculate_index(occupancy.get_value(), params);
            Bitboard::from_value(
                *slider_attacks::ATTACK_TABLE.get_unchecked(params.offset as usize + idx),
            )
        }
    }

    #[inline(always)]
    pub fn get_rook_attacks(square: Square, occupancy: Bitboard) -> Bitboard {
        unsafe {
            let params = &slider_attacks::LOOKUP[square.get_value() as usize];
            let idx = slider_attacks::calculate_index(occupancy.get_value(), params);
            Bitboard::from_value(
                *slider_attacks::ATTACK_TABLE.get_unchecked(params.offset as usize + idx),
            )
        }
    }
}
