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
    Attacks, Bitboard, ChessBoard, Move, MoveFlag, Piece, Side, move_gen::generate_moves::MoveGen,
};

pub(super) const KNIGHT: u8 = 0;
pub(super) const BISHOP: u8 = 1;
pub(super) const ROOK: u8 = 2;

impl MoveGen {
    pub fn generate_piece_moves<
        F: FnMut(Move),
        const COLOR: u8,
        const PIECE_TYPE: u8,
        const CAPTURE_ONLY: bool,
    >(
        board: &ChessBoard,
        push_map: Bitboard,
        capture_map: Bitboard,
        bishop_pins: Bitboard,
        rook_pins: Bitboard,
        apply_move: &mut F,
    ) {
        let pieces = match PIECE_TYPE {
            KNIGHT => {
                board.piece_mask_for_side(Piece::KNIGHT, Side::from(COLOR))
                    & !bishop_pins
                    & !rook_pins
            }
            BISHOP => {
                (board.piece_mask_for_side(Piece::BISHOP, Side::from(COLOR))
                    | board.piece_mask_for_side(Piece::QUEEN, Side::from(COLOR)))
                    & !rook_pins
            }
            ROOK => {
                (board.piece_mask_for_side(Piece::ROOK, Side::from(COLOR))
                    | board.piece_mask_for_side(Piece::QUEEN, Side::from(COLOR)))
                    & !bishop_pins
            }
            _ => unreachable!(),
        };

        let pinned_pieces = match PIECE_TYPE {
            KNIGHT => Bitboard::EMPTY,
            BISHOP => pieces & bishop_pins,
            ROOK => pieces & rook_pins,
            _ => unreachable!(),
        };

        let not_pinned_pieces = pieces & !pinned_pieces;

        not_pinned_pieces.map(|piece_square| {
            let attacks = match PIECE_TYPE {
                KNIGHT => Attacks::get_knight_attacks(piece_square),
                BISHOP => Attacks::get_bishop_attacks(piece_square, board.occupancy()),
                ROOK => Attacks::get_rook_attacks(piece_square, board.occupancy()),
                _ => unreachable!(),
            };

            (attacks & capture_map).map(|to_square| {
                apply_move(Move::from_squares(
                    piece_square,
                    to_square,
                    MoveFlag::CAPTURE,
                ))
            });

            if CAPTURE_ONLY {
                return;
            }

            (attacks & push_map).map(|to_square| {
                apply_move(Move::from_squares(
                    piece_square,
                    to_square,
                    MoveFlag::QUIET_MOVE,
                ))
            });
        });

        pinned_pieces.map(|piece_square| {
            let attacks = match PIECE_TYPE {
                KNIGHT => Bitboard::EMPTY,
                BISHOP => {
                    Attacks::get_bishop_attacks(piece_square, board.occupancy()) & bishop_pins
                }
                ROOK => Attacks::get_rook_attacks(piece_square, board.occupancy()) & rook_pins,
                _ => unreachable!(),
            };

            (attacks & capture_map).map(|to_square| {
                apply_move(Move::from_squares(
                    piece_square,
                    to_square,
                    MoveFlag::CAPTURE,
                ))
            });

            if CAPTURE_ONLY {
                return;
            }

            (attacks & push_map).map(|to_square| {
                apply_move(Move::from_squares(
                    piece_square,
                    to_square,
                    MoveFlag::QUIET_MOVE,
                ))
            });
        });
    }

    pub fn count_piece_moves<const COLOR: u8, const PIECE_TYPE: u8>(
        board: &ChessBoard,
        push_map: Bitboard,
        capture_map: Bitboard,
        bishop_pins: Bitboard,
        rook_pins: Bitboard,
    ) -> u32 {
        let mut result = 0;
        let pieces = match PIECE_TYPE {
            KNIGHT => {
                board.piece_mask_for_side(Piece::KNIGHT, Side::from(COLOR))
                    & !bishop_pins
                    & !rook_pins
            }
            BISHOP => {
                (board.piece_mask_for_side(Piece::BISHOP, Side::from(COLOR))
                    | board.piece_mask_for_side(Piece::QUEEN, Side::from(COLOR)))
                    & !rook_pins
            }
            ROOK => {
                (board.piece_mask_for_side(Piece::ROOK, Side::from(COLOR))
                    | board.piece_mask_for_side(Piece::QUEEN, Side::from(COLOR)))
                    & !bishop_pins
            }
            _ => unreachable!(),
        };

        let pinned_pieces = match PIECE_TYPE {
            KNIGHT => Bitboard::EMPTY,
            BISHOP => pieces & bishop_pins,
            ROOK => pieces & rook_pins,
            _ => unreachable!(),
        };

        let not_pinned_pieces = pieces & !pinned_pieces;

        let target_map = capture_map | push_map;

        not_pinned_pieces.map(|piece_square| {
            let attacks = match PIECE_TYPE {
                KNIGHT => Attacks::get_knight_attacks(piece_square),
                BISHOP => Attacks::get_bishop_attacks(piece_square, board.occupancy()),
                ROOK => Attacks::get_rook_attacks(piece_square, board.occupancy()),
                _ => unreachable!(),
            };

            result += (attacks & target_map).pop_count();
        });

        pinned_pieces.map(|piece_square| {
            let attacks = match PIECE_TYPE {
                KNIGHT => Bitboard::EMPTY,
                BISHOP => {
                    Attacks::get_bishop_attacks(piece_square, board.occupancy()) & bishop_pins
                }
                ROOK => Attacks::get_rook_attacks(piece_square, board.occupancy()) & rook_pins,
                _ => unreachable!(),
            };

            result += (attacks & target_map).pop_count();
        });

        result
    }
}
