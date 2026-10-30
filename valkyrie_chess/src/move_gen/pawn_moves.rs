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
    Attacks, Bitboard, ChessBoard, Move, MoveFlag, Piece, Side, Square,
    move_gen::generate_moves::{MoveGen, WHITE},
};

impl MoveGen {
    pub fn generate_pawn_moves<F: FnMut(Move), const COLOR: u8, const CAPTURE_ONLY: bool>(
        board: &ChessBoard,
        push_map: Bitboard,
        capture_map: Bitboard,
        bishop_pins: Bitboard,
        rook_pins: Bitboard,
        apply_move: &mut F,
    ) {
        let pawns = board.piece_mask_for_side(Piece::PAWN, Side::from(COLOR));

        handle_pawn_captures::<_, COLOR>(pawns & !rook_pins, capture_map, bishop_pins, apply_move);

        if board.en_passant_square() != Square::NULL {
            handle_en_passant::<_, COLOR>(board, pawns, apply_move);
        }

        if CAPTURE_ONLY {
            return;
        }

        handle_pawn_pushes::<_, COLOR>(
            board,
            pawns & !bishop_pins,
            push_map,
            rook_pins,
            apply_move,
        );
    }

    pub fn count_pawn_moves<const COLOR: u8>(
        board: &ChessBoard,
        push_map: Bitboard,
        capture_map: Bitboard,
        bishop_pins: Bitboard,
        rook_pins: Bitboard,
    ) -> u32 {
        let pawns = board.piece_mask_for_side(Piece::PAWN, Side::from(COLOR));
        let mut result = 0;

        result += count_pawn_captures::<COLOR>(pawns & !rook_pins, capture_map, bishop_pins);

        if board.en_passant_square() != Square::NULL {
            result += count_en_passant::<COLOR>(board, pawns);
        }

        result += count_pawn_pushes::<COLOR>(board, pawns & !bishop_pins, push_map, rook_pins);

        result
    }
}

fn handle_pawn_pushes<F: FnMut(Move), const COLOR: u8>(
    board: &ChessBoard,
    pawns: Bitboard,
    push_map: Bitboard,
    rook_pins: Bitboard,
    apply_move: &mut F,
) {
    let vertical_pin = rook_pins & (rook_pins << 8);
    let vertical_pin = vertical_pin | rook_pins >> 8;

    let promotion_rank = if COLOR == WHITE {
        Bitboard::RANK_7
    } else {
        Bitboard::RANK_2
    };
    let double_push_rank = if COLOR == WHITE {
        Bitboard::RANK_2
    } else {
        Bitboard::RANK_7
    };

    let moveable_pawns = pawns & !promotion_rank;
    let moveable_pawns = (moveable_pawns & !rook_pins) | (moveable_pawns & vertical_pin);

    let single_push_map = if COLOR == WHITE {
        push_map >> 8
    } else {
        push_map << 8
    };

    let mut single_push_pawns = moveable_pawns & single_push_map;
    let mut targets = if COLOR == WHITE {
        single_push_pawns << 8
    } else {
        single_push_pawns >> 8
    };
    while single_push_pawns.is_not_empty() {
        apply_move(Move::from_squares(
            single_push_pawns.pop_ls1b_square(),
            targets.pop_ls1b_square(),
            MoveFlag::QUIET_MOVE,
        ))
    }

    let double_push_map = if COLOR == WHITE {
        push_map >> 16
    } else {
        push_map << 16
    };
    let empty_squares_shifted_map = if COLOR == WHITE {
        !board.occupancy() >> 8
    } else {
        !board.occupancy() << 8
    };

    let mut double_push_pawns =
        moveable_pawns & double_push_rank & empty_squares_shifted_map & double_push_map;
    let mut targets = if COLOR == WHITE {
        double_push_pawns << 16
    } else {
        double_push_pawns >> 16
    };
    while double_push_pawns.is_not_empty() {
        apply_move(Move::from_squares(
            double_push_pawns.pop_ls1b_square(),
            targets.pop_ls1b_square(),
            MoveFlag::DOUBLE_PUSH,
        ))
    }

    let promotion_pawns = pawns & promotion_rank & !rook_pins;
    let targets = if COLOR == WHITE {
        promotion_pawns << 8
    } else {
        promotion_pawns >> 8
    } & push_map;
    targets.map(|to_square| {
        let from_square = if COLOR == WHITE {
            to_square >> 8
        } else {
            to_square << 8
        };
        apply_move(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::QUEEN_PROMOTION,
        ));
        apply_move(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::ROOK_PROMOTION,
        ));
        apply_move(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::BISHOP_PROMOTION,
        ));
        apply_move(Move::from_squares(
            from_square,
            to_square,
            MoveFlag::KNIGHT_PROMOTION,
        ));
    });
}

fn handle_pawn_captures<F: FnMut(Move), const COLOR: u8>(
    mut pawns: Bitboard,
    capture_map: Bitboard,
    bishop_pins: Bitboard,
    apply_move: &mut F,
) {
    let promotion_rank = if COLOR == WHITE {
        Bitboard::RANK_7
    } else {
        Bitboard::RANK_2
    };
    let promotion_pawns = pawns & promotion_rank;
    pawns &= !promotion_rank;

    (pawns & !bishop_pins).map(|from_square| {
        let attacks = Attacks::get_pawn_attacks(from_square, Side::from(COLOR)) & capture_map;
        attacks.map(|to_square| {
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::CAPTURE,
            ))
        });
    });

    (pawns & bishop_pins).map(|from_square| {
        let attacks =
            Attacks::get_pawn_attacks(from_square, Side::from(COLOR)) & capture_map & bishop_pins;
        attacks.map(|to_square| {
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::CAPTURE,
            ))
        });
    });

    (promotion_pawns & !bishop_pins).map(|from_square| {
        let attacks = Attacks::get_pawn_attacks(from_square, Side::from(COLOR)) & capture_map;
        attacks.map(|to_square| {
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::QUEEN_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::ROOK_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::BISHOP_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::KNIGHT_PROMOTION_CAPTURE,
            ));
        });
    });

    (promotion_pawns & bishop_pins).map(|from_square| {
        let attacks =
            Attacks::get_pawn_attacks(from_square, Side::from(COLOR)) & capture_map & bishop_pins;
        attacks.map(|to_square| {
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::QUEEN_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::ROOK_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::BISHOP_PROMOTION_CAPTURE,
            ));
            apply_move(Move::from_squares(
                from_square,
                to_square,
                MoveFlag::KNIGHT_PROMOTION_CAPTURE,
            ));
        });
    });
}

fn handle_en_passant<F: FnMut(Move), const COLOR: u8>(
    board: &ChessBoard,
    mut pawns: Bitboard,
    apply_move: &mut F,
) {
    let en_passant_square = board.en_passant_square();
    pawns &= Attacks::get_pawn_attacks(en_passant_square, Side::from(COLOR).flipped());

    pawns.map(|from_square| {
        if is_legal_en_passant::<COLOR>(board, from_square) {
            apply_move(Move::from_squares(
                from_square,
                en_passant_square,
                MoveFlag::EN_PASSANT,
            ))
        }
    });
}

fn count_pawn_pushes<const COLOR: u8>(
    board: &ChessBoard,
    pawns: Bitboard,
    push_map: Bitboard,
    rook_pins: Bitboard,
) -> u32 {
    let mut result = 0;
    let vertical_pin = rook_pins & (rook_pins << 8);
    let vertical_pin = vertical_pin | rook_pins >> 8;

    let promotion_rank = if COLOR == WHITE {
        Bitboard::RANK_7
    } else {
        Bitboard::RANK_2
    };
    let double_push_rank = if COLOR == WHITE {
        Bitboard::RANK_2
    } else {
        Bitboard::RANK_7
    };

    let moveable_pawns = pawns & !promotion_rank;
    let moveable_pawns = (moveable_pawns & !rook_pins) | (moveable_pawns & vertical_pin);

    let single_push_map = if COLOR == WHITE {
        push_map >> 8
    } else {
        push_map << 8
    };

    let single_push_pawns = moveable_pawns & single_push_map;
    result += single_push_pawns.pop_count();

    let double_push_map = if COLOR == WHITE {
        push_map >> 16
    } else {
        push_map << 16
    };
    let empty_squares_shifted_map = if COLOR == WHITE {
        !board.occupancy() >> 8
    } else {
        !board.occupancy() << 8
    };

    let double_push_pawns =
        moveable_pawns & double_push_rank & empty_squares_shifted_map & double_push_map;
    result += double_push_pawns.pop_count();

    let promotion_pawns = pawns & promotion_rank & !rook_pins;
    let targets = if COLOR == WHITE {
        promotion_pawns << 8
    } else {
        promotion_pawns >> 8
    } & push_map;
    result += targets.pop_count() * 4;

    result
}

fn count_pawn_captures<const COLOR: u8>(
    mut pawns: Bitboard,
    capture_map: Bitboard,
    bishop_pins: Bitboard,
) -> u32 {
    let promotion_rank = if COLOR == WHITE {
        Bitboard::RANK_7
    } else {
        Bitboard::RANK_2
    };
    let promotion_pawns = pawns & promotion_rank;
    pawns &= !promotion_rank;

    let mut result = 0;

    let non_pinned = pawns & !bishop_pins;
    if COLOR == WHITE {
        result += ((non_pinned << 7) & !Bitboard::FILE_H & capture_map).pop_count();
        result += ((non_pinned << 9) & !Bitboard::FILE_A & capture_map).pop_count();
    } else {
        result += ((non_pinned >> 9) & !Bitboard::FILE_H & capture_map).pop_count();
        result += ((non_pinned >> 7) & !Bitboard::FILE_A & capture_map).pop_count();
    }

    (pawns & bishop_pins).map(|from_square| {
        let attacks =
            Attacks::get_pawn_attacks(from_square, Side::from(COLOR)) & capture_map & bishop_pins;
        result += attacks.pop_count();
    });

    let non_pinned_promo = promotion_pawns & !bishop_pins;
    if COLOR == WHITE {
        result += ((non_pinned_promo << 7) & !Bitboard::FILE_H & capture_map).pop_count() * 4;
        result += ((non_pinned_promo << 9) & !Bitboard::FILE_A & capture_map).pop_count() * 4;
    } else {
        result += ((non_pinned_promo >> 9) & !Bitboard::FILE_H & capture_map).pop_count() * 4;
        result += ((non_pinned_promo >> 7) & !Bitboard::FILE_A & capture_map).pop_count() * 4;
    }

    (promotion_pawns & bishop_pins).map(|from_square| {
        let attacks =
            Attacks::get_pawn_attacks(from_square, Side::from(COLOR)) & capture_map & bishop_pins;
        result += attacks.pop_count() * 4;
    });

    result
}

fn count_en_passant<const COLOR: u8>(board: &ChessBoard, mut pawns: Bitboard) -> u32 {
    let en_passant_square = board.en_passant_square();
    pawns &= Attacks::get_pawn_attacks(en_passant_square, Side::from(COLOR).flipped());

    let mut result = 0;

    pawns.map(|from_square| {
        result += u32::from(is_legal_en_passant::<COLOR>(board, from_square));
    });

    result
}

fn is_legal_en_passant<const COLOR: u8>(board: &ChessBoard, from_square: Square) -> bool {
    let side = Side::from(COLOR);
    let en_passant_square = board.en_passant_square();
    let captured_square = if COLOR == WHITE {
        en_passant_square >> 8
    } else {
        en_passant_square << 8
    };
    let occupancy = board.occupancy()
        ^ Bitboard::from(from_square)
        ^ Bitboard::from(captured_square)
        ^ Bitboard::from(en_passant_square);

    (board.all_attackers_to_square_for_side(occupancy, board.king_square(side), side) & occupancy)
        .is_empty()
}
