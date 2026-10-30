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
    Bitboard, ChessBoard, Move, Side, Square,
    attacks::Rays,
    move_gen::piece_moves::{BISHOP, KNIGHT, ROOK},
};

pub(super) const WHITE: u8 = 0;
pub(super) const BLACK: u8 = 1;

pub(super) struct MoveGen;

impl ChessBoard {
    #[inline]
    pub fn map_legal_moves<F: FnMut(Move)>(&self, mut apply_move: F) {
        if self.side() == Side::WHITE {
            self.map_legal_moves_internal::<_, WHITE, false>(&mut apply_move)
        } else {
            self.map_legal_moves_internal::<_, BLACK, false>(&mut apply_move)
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

    #[inline]
    pub fn count_legal_moves(&self) -> u32 {
        if self.side() == Side::WHITE {
            self.count_legal_moves_templated::<WHITE>()
        } else {
            self.count_legal_moves_templated::<BLACK>()
        }
    }

    #[inline]
    pub fn has_legal_moves(&self) -> bool {
        if self.side() == Side::WHITE {
            self.has_legal_moves_templated::<WHITE>()
        } else {
            self.has_legal_moves_templated::<BLACK>()
        }
    }

    #[inline]
    pub(crate) fn map_legal_moves_templated<F: FnMut(Move), const COLOR: u8>(
        &self,
        mut apply_move: F,
    ) {
        self.map_legal_moves_internal::<_, COLOR, false>(&mut apply_move)
    }

    fn map_legal_moves_internal<F: FnMut(Move), const COLOR: u8, const CAPTURE_ONLY: bool>(
        &self,
        apply_move: &mut F,
    ) {
        let side = Side::from(COLOR);
        let attack_map = self.generate_attack_map(side.flipped());
        let king_square = self.king_square(side);

        MoveGen::generate_king_moves::<_, COLOR, CAPTURE_ONLY>(
            self,
            attack_map,
            king_square,
            apply_move,
        );

        let Some((push_map, capture_map)) = self.target_masks(side, attack_map, king_square) else {
            return;
        };
        let (bishop_pins, rook_pins) = self.generate_pin_masks(side);

        if !CAPTURE_ONLY && !attack_map.get_bit(king_square) {
            MoveGen::generate_castle_moves::<_, COLOR>(
                self,
                attack_map,
                king_square,
                rook_pins,
                apply_move,
            );
        }

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
    }

    pub(crate) fn count_legal_moves_templated<const COLOR: u8>(&self) -> u32 {
        let side = Side::from(COLOR);
        let attack_map = self.generate_attack_map(side.flipped());
        let king_square = self.king_square(side);
        let king_moves = MoveGen::count_king_moves::<COLOR>(self, attack_map, king_square);

        let Some((push_map, capture_map)) = self.target_masks(side, attack_map, king_square) else {
            return king_moves;
        };
        let (bishop_pins, rook_pins) = self.generate_pin_masks(side);

        let castle_moves = if attack_map.get_bit(king_square) {
            0
        } else {
            MoveGen::count_castle_moves::<COLOR>(self, attack_map, king_square, rook_pins)
        };

        king_moves
            + castle_moves
            + MoveGen::count_pawn_moves::<COLOR>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            )
            + MoveGen::count_piece_moves::<COLOR, { KNIGHT }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            )
            + MoveGen::count_piece_moves::<COLOR, { BISHOP }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            )
            + MoveGen::count_piece_moves::<COLOR, { ROOK }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            )
    }

    fn has_legal_moves_templated<const COLOR: u8>(&self) -> bool {
        let side = Side::from(COLOR);
        let attack_map = self.generate_attack_map(side.flipped());
        let king_square = self.king_square(side);

        if MoveGen::count_king_moves::<COLOR>(self, attack_map, king_square) > 0 {
            return true;
        }

        let Some((push_map, capture_map)) = self.target_masks(side, attack_map, king_square) else {
            return false;
        };
        let (bishop_pins, rook_pins) = self.generate_pin_masks(side);

        MoveGen::count_pawn_moves::<COLOR>(self, push_map, capture_map, bishop_pins, rook_pins) > 0
            || MoveGen::count_piece_moves::<COLOR, { KNIGHT }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            ) > 0
            || MoveGen::count_piece_moves::<COLOR, { BISHOP }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            ) > 0
            || MoveGen::count_piece_moves::<COLOR, { ROOK }>(
                self,
                push_map,
                capture_map,
                bishop_pins,
                rook_pins,
            ) > 0
            || (!attack_map.get_bit(king_square)
                && MoveGen::count_castle_moves::<COLOR>(self, attack_map, king_square, rook_pins)
                    > 0)
    }

    fn target_masks(
        &self,
        side: Side,
        attack_map: Bitboard,
        king_square: Square,
    ) -> Option<(Bitboard, Bitboard)> {
        if !attack_map.get_bit(king_square) {
            return Some((!self.occupancy(), self.occupancy_for_side(side.flipped())));
        }

        let checkers = self.generate_checkers_mask(side);
        let checker = checkers.ls1b_square();

        checkers.only_one_bit().then(|| {
            (
                Rays::get_ray(king_square, checker).exclude(checker),
                checkers,
            )
        })
    }
}
