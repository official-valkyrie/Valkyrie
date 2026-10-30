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

mod attacks;
mod base_structures;
mod board;
mod move_gen;

use std::time::Duration;
use std::time::Instant;

pub use attacks::Attacks;
pub use attacks::Rays;
pub use base_structures::Bitboard;
pub use base_structures::FEN;
pub use base_structures::FenViolation;
pub use base_structures::GameState;
pub use base_structures::Move;
pub use base_structures::MoveFlag;
pub use base_structures::Piece;
pub use base_structures::Side;
pub use base_structures::Square;
pub use base_structures::ZobristKey;
pub use board::ChessBoard;
pub use board::ChessPosition;

pub const DEFAULT_PERFT_DEPTH: u8 = 5;
pub const HALF_MOVE_LIMIT: u8 = 100;

pub fn perft<const BULK: bool>(
    board: &ChessBoard,
    depth: Option<u8>,
    mut on_root_move: impl FnMut(Move, u64),
) -> (u64, Duration) {
    let timer = Instant::now();
    let depth = depth.unwrap_or(DEFAULT_PERFT_DEPTH);

    if depth == 0 {
        return (1, timer.elapsed());
    }

    let mask = board.castle_rights().get_castle_mask();
    let count_child = if board.side() == Side::WHITE {
        perft_internal_black::<BULK>
    } else {
        perft_internal_white::<BULK>
    };
    let mut node_count = 0u64;

    board.map_legal_moves(|mv| {
        let mut board_copy = *board;
        board_copy.make_move_with_mask(mv, &mask);
        let result = count_child(&board_copy, depth - 1, &mask);
        node_count += result;
        on_root_move(mv, result);
    });

    (node_count, timer.elapsed())
}

fn perft_internal_white<const BULK: bool>(board: &ChessBoard, depth: u8, mask: &[u8; 64]) -> u64 {
    let mut node_count = 0u64;

    if depth == 0 {
        return 1;
    }

    if BULK && depth == 1 {
        return board.count_legal_moves_templated::<0>() as u64;
    }

    board.map_legal_moves_templated::<_, 0>(|mv| {
        let mut board_copy = *board;
        board_copy.make_move_templated::<0>(mv, mask);
        node_count += perft_internal_black::<BULK>(&board_copy, depth - 1, mask);
    });

    node_count
}

fn perft_internal_black<const BULK: bool>(board: &ChessBoard, depth: u8, mask: &[u8; 64]) -> u64 {
    let mut node_count = 0u64;

    if depth == 0 {
        return 1;
    }

    if BULK && depth == 1 {
        return board.count_legal_moves_templated::<1>() as u64;
    }

    board.map_legal_moves_templated::<_, 1>(|mv| {
        let mut board_copy = *board;
        board_copy.make_move_templated::<1>(mv, mask);
        node_count += perft_internal_white::<BULK>(&board_copy, depth - 1, mask);
    });

    node_count
}
