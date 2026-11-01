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

use valkyrie_chess::{Move, MoveFlag, Piece, Square};

#[test]
fn to_string() {
    assert_eq!(Move::NULL.to_string(false), "a1a1");
    assert_eq!(
        Move::from_squares(Square::A4, Square::B5, 0).to_string(false),
        "a4b5"
    );
    assert_eq!(
        Move::from_squares(Square::E7, Square::E8, MoveFlag::ROOK_PROMOTION).to_string(false),
        "e7e8r"
    );
    assert_eq!(
        Move::from_squares(Square::E1, Square::H1, MoveFlag::KING_SIDE_CASTLE).to_string(false),
        "e1g1"
    );
    assert_eq!(
        Move::from_squares(Square::E1, Square::A1, MoveFlag::QUEEN_SIDE_CASTLE).to_string(false),
        "e1c1"
    );
    assert_eq!(
        Move::from_squares(Square::E1, Square::H1, MoveFlag::KING_SIDE_CASTLE).to_string(true),
        "e1h1"
    );
    assert_eq!(
        Move::from_squares(Square::E1, Square::A1, MoveFlag::QUEEN_SIDE_CASTLE).to_string(true),
        "e1a1"
    );
}

#[test]
fn is_capture() {
    assert!(!Move::NULL.is_capture());
    assert!(Move::from_squares(Square::A1, Square::A1, MoveFlag::EN_PASSANT).is_capture());
    assert!(!Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION).is_capture());
    assert!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION_CAPTURE).is_capture()
    );
    assert!(Move::from_squares(Square::A1, Square::A1, MoveFlag::CAPTURE).is_capture());
    assert!(!Move::from_squares(Square::A1, Square::A1, MoveFlag::KING_SIDE_CASTLE).is_capture());
}

#[test]
fn is_promotion() {
    assert!(!Move::NULL.is_promotion());
    assert!(!Move::from_squares(Square::A1, Square::A1, MoveFlag::EN_PASSANT).is_promotion());
    assert!(Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION).is_promotion());
    assert!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION_CAPTURE).is_promotion()
    );
    assert!(!Move::from_squares(Square::A1, Square::A1, MoveFlag::CAPTURE).is_promotion());
    assert!(!Move::from_squares(Square::A1, Square::A1, MoveFlag::KING_SIDE_CASTLE).is_promotion());
}

#[test]
fn promotion_piece() {
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::KNIGHT_PROMOTION).promotion_piece(),
        Piece::KNIGHT
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::BISHOP_PROMOTION).promotion_piece(),
        Piece::BISHOP
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION).promotion_piece(),
        Piece::ROOK
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::QUEEN_PROMOTION).promotion_piece(),
        Piece::QUEEN
    );

    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::KNIGHT_PROMOTION_CAPTURE)
            .promotion_piece(),
        Piece::KNIGHT
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::BISHOP_PROMOTION_CAPTURE)
            .promotion_piece(),
        Piece::BISHOP
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::ROOK_PROMOTION_CAPTURE)
            .promotion_piece(),
        Piece::ROOK
    );
    assert_eq!(
        Move::from_squares(Square::A1, Square::A1, MoveFlag::QUEEN_PROMOTION_CAPTURE)
            .promotion_piece(),
        Piece::QUEEN
    );
}
