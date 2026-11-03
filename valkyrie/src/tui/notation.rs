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

use valkyrie_chess::{ChessBoard, Move, MoveFlag, Square};

const PIECE_LETTERS: [&str; 6] = ["", "N", "B", "R", "Q", "K"];

pub fn san_line(board: &ChessBoard, moves: &[Move]) -> Vec<String> {
    let mut board = *board;

    moves
        .iter()
        .map(|&mv| {
            let text = san(&board, mv);
            board.make_move(mv);
            text
        })
        .collect()
}

pub fn san(board: &ChessBoard, mv: Move) -> String {
    let mut text = body(board, mv);
    let mut next = *board;

    next.make_move(mv);

    if next.is_in_check() {
        text.push(if next.count_legal_moves() == 0 {
            '#'
        } else {
            '+'
        });
    }

    text
}

fn body(board: &ChessBoard, mv: Move) -> String {
    match mv.flag() {
        MoveFlag::KING_SIDE_CASTLE => return "O-O".to_string(),
        MoveFlag::QUEEN_SIDE_CASTLE => return "O-O-O".to_string(),
        _ => {}
    }

    let from = mv.from_square();
    let to = square_name(mv.to_square());
    let piece = usize::from(board.piece_on_square(from));
    let capture = if mv.is_capture() { "x" } else { "" };

    if piece == 0 {
        let file = if mv.is_capture() {
            &square_name(from)[..1]
        } else {
            ""
        };
        let promotion = if mv.is_promotion() {
            format!("={}", PIECE_LETTERS[usize::from(mv.promotion_piece())])
        } else {
            String::new()
        };

        return format!("{file}{capture}{to}{promotion}");
    }

    format!(
        "{}{}{capture}{to}",
        PIECE_LETTERS[piece],
        disambiguation(board, mv, piece)
    )
}

fn disambiguation(board: &ChessBoard, mv: Move, piece: usize) -> String {
    let from = mv.from_square();
    let mut rivals = Vec::new();

    board.map_legal_moves(|other| {
        if other.to_square() == mv.to_square()
            && other.from_square() != from
            && !other.is_castle()
            && usize::from(board.piece_on_square(other.from_square())) == piece
        {
            rivals.push(other.from_square());
        }
    });

    if rivals.is_empty() {
        return String::new();
    }

    let name = square_name(from);
    let (file, rank) = (u8::from(from) % 8, u8::from(from) / 8);

    if rivals.iter().all(|rival| u8::from(*rival) % 8 != file) {
        name[..1].to_string()
    } else if rivals.iter().all(|rival| u8::from(*rival) / 8 != rank) {
        name[1..].to_string()
    } else {
        name
    }
}

fn square_name(square: Square) -> String {
    square.to_string()
}
