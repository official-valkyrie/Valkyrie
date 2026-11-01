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
    Square,
    base_structures::{Piece, Side},
};

const SQUARE_MASK: u16 = 0b11_1111;
const FLAG_SHIFT: u16 = 6;
const FLAG_MASK: u16 = 0b1111 << FLAG_SHIFT;
const TO_SQUARE_SHIFT: u16 = 10;

pub struct MoveFlag;
impl MoveFlag {
    pub const QUIET_MOVE: u16 = 0b0000 << FLAG_SHIFT;
    pub const DOUBLE_PUSH: u16 = 0b0001 << FLAG_SHIFT;
    pub const KING_SIDE_CASTLE: u16 = 0b0010 << FLAG_SHIFT;
    pub const QUEEN_SIDE_CASTLE: u16 = 0b0011 << FLAG_SHIFT;
    pub const CAPTURE: u16 = 0b0100 << FLAG_SHIFT;
    pub const EN_PASSANT: u16 = 0b0101 << FLAG_SHIFT;
    pub const KNIGHT_PROMOTION: u16 = 0b1000 << FLAG_SHIFT;
    pub const BISHOP_PROMOTION: u16 = 0b1001 << FLAG_SHIFT;
    pub const ROOK_PROMOTION: u16 = 0b1010 << FLAG_SHIFT;
    pub const QUEEN_PROMOTION: u16 = 0b1011 << FLAG_SHIFT;
    pub const KNIGHT_PROMOTION_CAPTURE: u16 = 0b1100 << FLAG_SHIFT;
    pub const BISHOP_PROMOTION_CAPTURE: u16 = 0b1101 << FLAG_SHIFT;
    pub const ROOK_PROMOTION_CAPTURE: u16 = 0b1110 << FLAG_SHIFT;
    pub const QUEEN_PROMOTION_CAPTURE: u16 = 0b1111 << FLAG_SHIFT;
}

#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Move(u16);
impl Move {
    pub const NULL: Self = Self(0);

    #[inline]
    pub fn from_squares(from_square: Square, to_square: Square, flag: u16) -> Self {
        Self(u16::from(to_square) << TO_SQUARE_SHIFT | flag | u16::from(from_square))
    }

    #[inline]
    pub fn from_square(&self) -> Square {
        Square::from((self.0 & SQUARE_MASK) as u8)
    }

    #[inline]
    pub fn to_square(&self) -> Square {
        Square::from((self.0 >> TO_SQUARE_SHIFT) as u8)
    }

    #[inline]
    pub fn flag(&self) -> u16 {
        self.0 & FLAG_MASK
    }

    #[inline]
    pub fn is_capture(&self) -> bool {
        self.0 & MoveFlag::CAPTURE != 0
    }

    #[inline]
    pub fn is_en_passant(&self) -> bool {
        self.flag() == MoveFlag::EN_PASSANT
    }

    #[inline]
    pub fn is_castle(&self) -> bool {
        let flag = self.flag();
        flag == MoveFlag::KING_SIDE_CASTLE || flag == MoveFlag::QUEEN_SIDE_CASTLE
    }

    #[inline]
    pub fn is_promotion(&self) -> bool {
        self.0 & MoveFlag::KNIGHT_PROMOTION != 0
    }

    #[inline]
    pub fn promotion_piece(&self) -> Piece {
        Piece::from((((self.flag() >> FLAG_SHIFT) & 3) + 1) as u8)
    }

    pub fn to_string(&self, chess960: bool) -> String {
        if !chess960 && self.is_castle() {
            let side = if u8::from(self.from_square()) < 32 {
                Side::WHITE
            } else {
                Side::BLACK
            };
            let destination_square = if self.flag() == MoveFlag::QUEEN_SIDE_CASTLE {
                Square::C1
            } else {
                Square::G1
            } + 56 * u8::from(side);
            return format!("{}{}", self.from_square(), destination_square);
        }

        format!(
            "{}{}{}",
            self.from_square(),
            self.to_square(),
            if self.is_promotion() {
                ["n", "b", "r", "q"][(u8::from(self.promotion_piece()) - 1) as usize]
            } else {
                ""
            }
        )
    }
}

impl From<Move> for u16 {
    #[inline]
    fn from(value: Move) -> Self {
        value.0
    }
}

impl From<u16> for Move {
    #[inline]
    fn from(value: u16) -> Self {
        Self(value)
    }
}
