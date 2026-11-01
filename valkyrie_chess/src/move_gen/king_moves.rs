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
    Attacks, Bitboard, ChessBoard, Move, MoveFlag, Side, Square,
    attacks::Rays,
    base_structures::CastleRights,
    move_gen::generate_moves::{MoveGen, WHITE},
};

impl MoveGen {
    pub fn generate_king_moves<F: FnMut(Move), const COLOR: u8, const CAPTURE_ONLY: bool>(
        board: &ChessBoard,
        attack_map: Bitboard,
        king_square: Square,
        apply_move: &mut F,
    ) {
        let move_mask = Attacks::get_king_attacks(king_square) & !attack_map;

        (move_mask & board.occupancy_for_side(Side::from(COLOR).flipped()))
            .map(|square| apply_move(Move::from_squares(king_square, square, MoveFlag::CAPTURE)));

        if CAPTURE_ONLY {
            return;
        }

        (move_mask & !board.occupancy()).map(|square| {
            apply_move(Move::from_squares(
                king_square,
                square,
                MoveFlag::QUIET_MOVE,
            ))
        });
    }

    pub fn generate_castle_moves<F: FnMut(Move), const COLOR: u8>(
        board: &ChessBoard,
        attack_map: Bitboard,
        king_square: Square,
        rook_pins: Bitboard,
        apply_move: &mut F,
    ) {
        let validate_castle =
            |rook_square: Square, king_destination: Square, rook_destination: Square| -> bool {
                let castle_path = Rays::get_ray(rook_square, rook_destination)
                    | Rays::get_ray(king_square, king_destination);
                let occupancy =
                    board.occupancy() ^ Bitboard::from(rook_square) ^ Bitboard::from(king_square);
                (Rays::get_ray(king_square, king_destination) & attack_map).is_empty()
                    && (castle_path & occupancy).is_empty()
                    && (rook_pins & Bitboard::from(rook_square)).is_empty()
            };

        if COLOR == WHITE {
            let rook_square = board.castle_rights().rook_square(1);
            if board.castle_rights().has_right(CastleRights::WHITE_KING)
                && validate_castle(rook_square, Square::G1, Square::F1)
            {
                apply_move(Move::from_squares(
                    king_square,
                    rook_square,
                    MoveFlag::KING_SIDE_CASTLE,
                ))
            }

            let rook_square = board.castle_rights().rook_square(0);
            if board.castle_rights().has_right(CastleRights::WHITE_QUEEN)
                && validate_castle(rook_square, Square::C1, Square::D1)
            {
                apply_move(Move::from_squares(
                    king_square,
                    rook_square,
                    MoveFlag::QUEEN_SIDE_CASTLE,
                ))
            }
        } else {
            let rook_square = board.castle_rights().rook_square(3);
            if board.castle_rights().has_right(CastleRights::BLACK_KING)
                && validate_castle(rook_square, Square::G8, Square::F8)
            {
                apply_move(Move::from_squares(
                    king_square,
                    rook_square,
                    MoveFlag::KING_SIDE_CASTLE,
                ))
            }

            let rook_square = board.castle_rights().rook_square(2);
            if board.castle_rights().has_right(CastleRights::BLACK_QUEEN)
                && validate_castle(rook_square, Square::C8, Square::D8)
            {
                apply_move(Move::from_squares(
                    king_square,
                    rook_square,
                    MoveFlag::QUEEN_SIDE_CASTLE,
                ))
            }
        }
    }

    pub fn count_king_moves<const COLOR: u8>(
        board: &ChessBoard,
        attack_map: Bitboard,
        king_square: Square,
    ) -> u32 {
        let move_mask = Attacks::get_king_attacks(king_square) & !attack_map;
        (move_mask & !board.occupancy_for_side(Side::from(COLOR))).pop_count()
    }

    pub fn count_castle_moves<const COLOR: u8>(
        board: &ChessBoard,
        attack_map: Bitboard,
        king_square: Square,
        rook_pins: Bitboard,
    ) -> u32 {
        let validate_castle =
            |rook_square: Square, king_destination: Square, rook_destination: Square| -> bool {
                let castle_path = Rays::get_ray(rook_square, rook_destination)
                    | Rays::get_ray(king_square, king_destination);
                let occupancy =
                    board.occupancy() ^ Bitboard::from(rook_square) ^ Bitboard::from(king_square);
                (Rays::get_ray(king_square, king_destination) & attack_map).is_empty()
                    && (castle_path & occupancy).is_empty()
                    && (rook_pins & Bitboard::from(rook_square)).is_empty()
            };

        let mut result = 0;

        if COLOR == WHITE {
            let rook_square = board.castle_rights().rook_square(1);
            if board.castle_rights().has_right(CastleRights::WHITE_KING)
                && validate_castle(rook_square, Square::G1, Square::F1)
            {
                result += 1;
            }

            let rook_square = board.castle_rights().rook_square(0);
            if board.castle_rights().has_right(CastleRights::WHITE_QUEEN)
                && validate_castle(rook_square, Square::C1, Square::D1)
            {
                result += 1;
            }
        } else {
            let rook_square = board.castle_rights().rook_square(3);
            if board.castle_rights().has_right(CastleRights::BLACK_KING)
                && validate_castle(rook_square, Square::G8, Square::F8)
            {
                result += 1;
            }

            let rook_square = board.castle_rights().rook_square(2);
            if board.castle_rights().has_right(CastleRights::BLACK_QUEEN)
                && validate_castle(rook_square, Square::C8, Square::D8)
            {
                result += 1;
            }
        };

        result
    }
}
