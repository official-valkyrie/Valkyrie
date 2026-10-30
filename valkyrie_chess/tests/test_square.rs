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

use valkyrie_chess::{Bitboard, InvalidSquare, Square};

#[test]
fn shift_left() {
    assert_eq!(Square::from(32) << 3, Square::from(35));
    assert_eq!(
        Square::from(32) << 3,
        Bitboard::from(Square::from(32)).shift_left(3).ls1b_square()
    );
}

#[test]
fn shift_right() {
    assert_eq!(Square::from(32) >> 3, Square::from(29));
    assert_eq!(
        Square::from(32) >> 3,
        Bitboard::from(Square::from(32))
            .shift_right(3)
            .ls1b_square()
    );
}

#[test]
fn from_string() {
    assert_eq!(Square::try_from("h8"), Ok(Square::H8));
    assert_eq!(Square::try_from("b2"), Ok(Square::B2));
    assert_eq!(Square::try_from("a1"), Ok(Square::A1));
}

#[test]
fn from_string_rejects_garbage() {
    for text in [
        "", "-", "e", "e9", "e0", "i1", "E4", "e44", "e4 ", "\u{e9}4", "NULL",
    ] {
        assert_eq!(text.parse::<Square>(), Err(InvalidSquare), "{text:?}");
    }
}

#[test]
fn to_string() {
    assert_eq!(String::from(Square::H8), "h8");
    assert_eq!(String::from(Square::B2), "b2");
    assert_eq!(String::from(Square::NULL), "NULL");
}
