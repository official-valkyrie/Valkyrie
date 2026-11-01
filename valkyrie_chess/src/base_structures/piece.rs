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

use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Piece(u8);
impl Piece {
    pub const PAWN: Self = Self(0);
    pub const KNIGHT: Self = Self(1);
    pub const BISHOP: Self = Self(2);
    pub const ROOK: Self = Self(3);
    pub const QUEEN: Self = Self(4);
    pub const KING: Self = Self(5);
    pub const NONE: Self = Self(u8::MAX);

    pub const fn value(&self) -> usize {
        self.0 as usize
    }
}

impl From<u8> for Piece {
    #[inline]
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<usize> for Piece {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value as u8)
    }
}

impl From<Piece> for u8 {
    #[inline]
    fn from(value: Piece) -> Self {
        value.0
    }
}

impl From<Piece> for usize {
    #[inline]
    fn from(value: Piece) -> Self {
        value.0 as usize
    }
}

impl From<Piece> for char {
    fn from(piece: Piece) -> Self {
        match piece {
            Piece::PAWN => 'p',
            Piece::KNIGHT => 'n',
            Piece::BISHOP => 'b',
            Piece::ROOK => 'r',
            Piece::QUEEN => 'q',
            Piece::KING => 'k',
            _ => ' ',
        }
    }
}

impl Display for Piece {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", char::from(*self))
    }
}
