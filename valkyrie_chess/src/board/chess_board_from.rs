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

use crate::{FEN, Piece, Side, Square, base_structures::CastleRights, board::ChessBoard};

impl From<&FEN> for ChessBoard {
    fn from(value: &FEN) -> Self {
        let mut board = Self::default();

        for (rank_index, rank) in value.board.clone().into_iter().enumerate() {
            let mut index = 0;
            let mut file = 0;
            while file < 8 {
                let square = Square::from_coords((7 - rank_index) as u8, file);
                let piece_char = rank.as_bytes()[index] as char;
                if piece_char.is_numeric() {
                    file += piece_char as u8 - b'0';
                    index += 1;
                    continue;
                }

                let side = Side::from(piece_char as u8 > 90);

                match piece_char.to_ascii_lowercase() {
                    'p' => board.set_piece_on_square(square, Piece::PAWN, side),
                    'n' => board.set_piece_on_square(square, Piece::KNIGHT, side),
                    'b' => board.set_piece_on_square(square, Piece::BISHOP, side),
                    'r' => board.set_piece_on_square(square, Piece::ROOK, side),
                    'q' => board.set_piece_on_square(square, Piece::QUEEN, side),
                    'k' => board.set_piece_on_square(square, Piece::KING, side),
                    _ => unreachable!(),
                }

                index += 1;
                file += 1;
            }
        }

        board.side = Side::from(value.side_to_move == "b");

        let kings = [
            board.king_square(Side::WHITE),
            board.king_square(Side::BLACK),
        ];
        let mut rooks = [Square::NULL; 4];
        let mut rights = 0u8;

        for right in value.castle_rights.chars() {
            let side = Side::from(right.is_ascii_lowercase());
            let Some((file, king_side)) = CastleRights::resolve(&value.home_rank(side), right)
            else {
                continue;
            };

            let index = 2 * usize::from(side) + usize::from(king_side);
            rights |= 0b1000 >> index;
            rooks[index] = Square::from_coords(board.king_square(side).get_rank(), file);
        }

        board.castle_rights = CastleRights::create_base(rooks, kings);
        board.castle_rights.set_rights(rights);

        if let Ok(square) = value.en_passant_square.parse::<Square>()
            && board.is_en_passant_capturable(square, board.side().flipped())
        {
            board.en_passant_square = square;
        }

        board.half_moves = value.half_move_counter.parse::<u8>().unwrap_or(0);

        board
    }
}

impl From<&ChessBoard> for FEN {
    fn from(value: &ChessBoard) -> Self {
        let mut fen = String::new();

        for rank in (0..8).rev() {
            let mut empty_count = 0;
            for file in 0..8 {
                let square = Square::from_coords(rank, file);
                let piece = value.piece_on_square(square);
                let side = value.color_on_square(square);
                if piece != Piece::NONE {
                    if empty_count > 0 {
                        fen.push_str(&empty_count.to_string());
                        empty_count = 0;
                    }
                    let piece_char = char::from(piece);
                    if side == Side::WHITE {
                        fen.push(piece_char.to_uppercase().next().unwrap());
                    } else {
                        fen.push(piece_char);
                    }
                } else {
                    empty_count += 1;
                }
            }
            if empty_count > 0 {
                fen.push_str(&empty_count.to_string());
            }
            if rank > 0 {
                fen.push('/');
            }
        }

        fen.push(' ');
        fen.push(if value.side() == Side::WHITE {
            'w'
        } else {
            'b'
        });

        fen.push(' ');
        fen.push_str(String::from(*value.castle_rights()).as_str());

        fen.push(' ');
        if value.en_passant_square() == Square::NULL {
            fen.push('-');
        } else {
            fen.push_str(String::from(value.en_passant_square()).as_str());
        }

        fen.push(' ');
        fen.push_str(&value.half_moves().to_string());
        fen.push(' ');
        fen.push_str(&(1).to_string());

        FEN::from(fen)
    }
}
