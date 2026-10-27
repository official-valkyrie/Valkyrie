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
    Bitboard, ChessBoard, Move, Side,
    attacks::Rays,
    move_gen::piece_moves::{BISHOP, KNIGHT, ROOK},
};

pub(super) const WHITE: u8 = 0;
pub(super) const BLACK: u8 = 1;

pub(super) struct MoveGen;

impl ChessBoard {
    #[inline]
    pub(crate) fn map_legal_moves_templated<F: FnMut(Move), const COLOR: u8>(
        &self,
        mut apply_move: F,
    ) {
        self.map_legal_moves_internal::<_, COLOR, false>(&mut apply_move)
    }

    #[inline]
    pub fn map_legal_moves<F: FnMut(Move)>(&self, mut apply_move: F) {
        if self.side() == Side::WHITE {
            self.map_legal_moves_internal::<_, WHITE, false>(&mut apply_move)
        } else {
            self.map_legal_moves_internal::<_, BLACK, false>(&mut apply_move)
        }
    }

    #[inline]
    pub fn has_legal_moves(&self) -> bool {
        if self.side() == Side::WHITE {
            self.count_legal_moves::<WHITE>() > 0
        } else {
            self.count_legal_moves::<BLACK>() > 0
        }
    }

    #[inline]
    pub fn map_capture_moves<F: FnMut(Move)>(&self, mut apply_move: F) {
        if self.side() == Side::WHITE {
            self.map_legal_moves_internal::<_, WHITE, true>(&mut apply_move)
        } else {
            self.map_legal_moves_internal::<_, BLACK, true>(&mut apply_move)
        }
    }

    pub fn map_legal_moves_internal<F: FnMut(Move), const COLOR: u8, const CAPTURE_ONLY: bool>(
        &self,
        apply_move: &mut F,
    ) {
        let attack_map = self.generate_attack_map(Side::from(COLOR).flipped());
        let king_square = self.king_square(Side::from(COLOR));
        let (bishop_pins, rook_pins) = self.generate_pin_masks(Side::from(COLOR));
        let checkers = if attack_map.get_bit(king_square) {
            self.generate_checkers_mask(Side::from(COLOR))
        } else {
            Bitboard::EMPTY
        };

        MoveGen::generate_king_moves::<_, COLOR, CAPTURE_ONLY>(
            self,
            attack_map,
            king_square,
            apply_move,
        );

        if checkers.is_empty() {
            if !CAPTURE_ONLY {
                MoveGen::generate_castle_moves::<_, COLOR>(
                    self,
                    attack_map,
                    king_square,
                    rook_pins,
                    apply_move,
                )
            }

            let push_map = !self.occupancy();
            let capture_map = self.occupancy_for_side(Side::from(COLOR).flipped());

            MoveGen::generate_pawn_moves::<_, COLOR, CAPTURE_ONLY>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { KNIGHT }, CAPTURE_ONLY>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { BISHOP }, CAPTURE_ONLY>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { ROOK }, CAPTURE_ONLY>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
                apply_move,
            );
        } else if (checkers & (checkers - 1)).is_empty() {
            let checker = checkers.ls1b_square();
            let push_map = Rays::get_ray(king_square, checker).exclude(checker);

            MoveGen::generate_pawn_moves::<_, COLOR, CAPTURE_ONLY>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { KNIGHT }, CAPTURE_ONLY>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { BISHOP }, CAPTURE_ONLY>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
                apply_move,
            );
            MoveGen::generate_piece_moves::<_, COLOR, { ROOK }, CAPTURE_ONLY>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
                apply_move,
            );
        }
    }

    pub fn count_legal_moves<const COLOR: u8>(&self) -> u32 {
        let attack_map = self.generate_attack_map(Side::from(COLOR).flipped());
        let king_square = self.king_square(Side::from(COLOR));
        let (bishop_pins, rook_pins) = self.generate_pin_masks(Side::from(COLOR));
        let checkers = if attack_map.get_bit(king_square) {
            self.generate_checkers_mask(Side::from(COLOR))
        } else {
            Bitboard::EMPTY
        };

        let mut result = 0;

        result += MoveGen::count_king_moves::<COLOR>(self, attack_map, king_square);

        if checkers.is_empty() {
            result +=
                MoveGen::count_castle_moves::<COLOR>(self, attack_map, king_square, rook_pins);

            let push_map = !self.occupancy();
            let capture_map = self.occupancy_for_side(Side::from(COLOR).flipped());

            result += MoveGen::count_pawn_moves::<COLOR>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { KNIGHT }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { BISHOP }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { ROOK }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            );
        } else if (checkers & (checkers - 1)).is_empty() {
            let checker = checkers.ls1b_square();
            let push_map = Rays::get_ray(king_square, checker).exclude(checker);

            result += MoveGen::count_pawn_moves::<COLOR>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { KNIGHT }>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { BISHOP }>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
            );
            result += MoveGen::count_piece_moves::<COLOR, { ROOK }>(
                self,
                push_map,
                checkers,
                bishop_pins,
                rook_pins,
            );
        };

        result
    }
}
