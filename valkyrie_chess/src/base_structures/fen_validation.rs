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

use std::fmt::{
    Display,
    Formatter,
    Result,
};

use crate::{
    ChessBoard,
    FEN,
    Piece,
    Side,
    Square,
};

const EMPTY: char = '.';

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FenViolation {
    MissingFields(usize),
    ExtraFields(usize),
    WrongRankCount(usize),
    InvalidRank(usize),
    KingCount(Side, usize),
    PawnCount(Side, usize),
    PieceCount(Side, usize),
    PawnOnBackRank,
    InvalidSideToMove,
    OpponentInCheck,
    InvalidCastlingRights,
    UnavailableCastlingRight(char),
    InvalidEnPassant,
    ImpossibleEnPassant,
    InvalidHalfMoveClock,
    InvalidFullMoveNumber,
}

impl Display for FenViolation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        match self {
            Self::MissingFields(found) => write!(formatter, "expected at least 4 fields, found {found}"),
            Self::ExtraFields(found) => write!(formatter, "expected at most 6 fields, found {found}"),
            Self::WrongRankCount(found) => write!(formatter, "expected 8 ranks, found {found}"),
            Self::InvalidRank(rank) => {
                write!(formatter, "rank {rank} must describe exactly 8 squares using pnbrqkPNBRQK and digits 1-8")
            }
            Self::KingCount(side, found) => write!(formatter, "expected 1 {} king, found {found}", side_name(*side)),
            Self::PawnCount(side, found) => {
                write!(formatter, "{} has {found} pawns, at most 8 are allowed", side_name(*side))
            }
            Self::PieceCount(side, found) => {
                write!(formatter, "{} has {found} pieces, at most 16 are allowed", side_name(*side))
            }
            Self::PawnOnBackRank => write!(formatter, "pawns can't stand on rank 1 or 8"),
            Self::InvalidSideToMove => write!(formatter, "side to move must be 'w' or 'b'"),
            Self::OpponentInCheck => write!(formatter, "the side not to move is in check"),
            Self::InvalidCastlingRights => write!(
                formatter,
                "castling rights must be '-' or up to two of KQ/A-H for white and kq/a-h for black, without repeats"
            ),
            Self::UnavailableCastlingRight(right) => {
                write!(formatter, "castling right '{right}' needs a king and a matching rook on their home rank")
            }
            Self::InvalidEnPassant => write!(
                formatter,
                "en passant must be '-' or a square on rank 6 with white to move, rank 3 with black to move"
            ),
            Self::ImpossibleEnPassant => write!(formatter, "no pawn could have just moved past the en passant square"),
            Self::InvalidHalfMoveClock => write!(formatter, "half-move clock must be a number from 0 to 255"),
            Self::InvalidFullMoveNumber => write!(formatter, "full-move number must be a number from 1 to 65535"),
        }
    }
}

impl FEN {
    pub fn validate_fen(fen_string: &str) -> Vec<FenViolation> {
        let fen_parts: Vec<&str> = fen_string.split_whitespace().collect();

        if fen_parts.len() < 4 {
            return vec![FenViolation::MissingFields(fen_parts.len())];
        }

        let mut violations = Vec::new();

        if fen_parts.len() > 6 {
            violations.push(FenViolation::ExtraFields(fen_parts.len()));
        }

        let squares = validate_placement(fen_parts[0], &mut violations);
        let side_to_move = validate_side_to_move(fen_parts[1], &mut violations);

        validate_castling_rights(fen_parts[2], squares.as_ref(), &mut violations);
        validate_en_passant(fen_parts[3], squares.as_ref(), side_to_move, &mut violations);

        if let (Some(squares), Some(side_to_move)) = (&squares, side_to_move)
            && count(squares, 'K') == 1
            && count(squares, 'k') == 1
            && is_opponent_in_check(squares, side_to_move)
        {
            violations.push(FenViolation::OpponentInCheck);
        }

        if fen_parts.get(4).is_some_and(|half_moves| half_moves.parse::<u8>().is_err()) {
            violations.push(FenViolation::InvalidHalfMoveClock);
        }

        if fen_parts.get(5).is_some_and(|full_moves| !full_moves.parse::<u16>().is_ok_and(|number| number > 0)) {
            violations.push(FenViolation::InvalidFullMoveNumber);
        }

        violations
    }
}

fn validate_placement(placement: &str, violations: &mut Vec<FenViolation>) -> Option<[char; 64]> {
    let ranks: Vec<&str> = placement.split('/').collect();
    let mut squares = [EMPTY; 64];
    let mut is_valid = ranks.len() == 8;

    if ranks.len() != 8 {
        violations.push(FenViolation::WrongRankCount(ranks.len()));
    } else {
        for (index, rank) in ranks.iter().enumerate() {
            let rank_number = 8 - index;

            match expand_rank(rank) {
                Some(row) => squares[(rank_number - 1) * 8..rank_number * 8].copy_from_slice(&row),
                None => {
                    violations.push(FenViolation::InvalidRank(rank_number));
                    is_valid = false;
                }
            }
        }
    }

    for (side, king) in [(Side::WHITE, 'K'), (Side::BLACK, 'k')] {
        let kings = placement.matches(king).count();
        if kings != 1 {
            violations.push(FenViolation::KingCount(side, kings));
        }
    }

    if !is_valid {
        return None;
    }

    if squares[..8].iter().chain(&squares[56..]).any(|&symbol| symbol == 'P' || symbol == 'p') {
        violations.push(FenViolation::PawnOnBackRank);
    }

    for (side, pawn) in [(Side::WHITE, 'P'), (Side::BLACK, 'p')] {
        let pawns = count(&squares, pawn);
        if pawns > 8 {
            violations.push(FenViolation::PawnCount(side, pawns));
        }

        let pieces = squares
            .iter()
            .filter(|&&symbol| symbol != EMPTY && symbol.is_ascii_uppercase() == (side == Side::WHITE))
            .count();
        if pieces > 16 {
            violations.push(FenViolation::PieceCount(side, pieces));
        }
    }

    Some(squares)
}

fn validate_side_to_move(field: &str, violations: &mut Vec<FenViolation>) -> Option<Side> {
    match field {
        "w" => Some(Side::WHITE),
        "b" => Some(Side::BLACK),
        _ => {
            violations.push(FenViolation::InvalidSideToMove);
            None
        }
    }
}

fn validate_castling_rights(field: &str, squares: Option<&[char; 64]>, violations: &mut Vec<FenViolation>) {
    if field == "-" {
        return;
    }

    let is_right = |symbol: char| matches!(symbol, 'K' | 'Q' | 'k' | 'q' | 'A'..='H' | 'a'..='h');
    let has_repeats = field.chars().enumerate().any(|(index, symbol)| field.chars().skip(index + 1).any(|other| other == symbol));
    let white_rights = field.chars().filter(char::is_ascii_uppercase).count();
    let black_rights = field.chars().filter(char::is_ascii_lowercase).count();

    if !field.chars().all(is_right) || has_repeats || white_rights > 2 || black_rights > 2 {
        violations.push(FenViolation::InvalidCastlingRights);
        return;
    }

    let Some(squares) = squares else {
        return;
    };

    for right in field.chars() {
        if !is_castling_available(squares, right) {
            violations.push(FenViolation::UnavailableCastlingRight(right));
        }
    }
}

fn is_castling_available(squares: &[char; 64], right: char) -> bool {
    let (king, rook, home_rank) = if right.is_ascii_uppercase() {
        ('K', 'R', &squares[..8])
    } else {
        ('k', 'r', &squares[56..])
    };

    let Some(king_file) = home_rank.iter().position(|&symbol| symbol == king) else {
        return false;
    };

    match right.to_ascii_lowercase() {
        'k' => home_rank[king_file + 1..].contains(&rook),
        'q' => home_rank[..king_file].contains(&rook),
        file => home_rank[usize::from(file as u8 - b'a')] == rook,
    }
}

fn validate_en_passant(
    field: &str,
    squares: Option<&[char; 64]>,
    side_to_move: Option<Side>,
    violations: &mut Vec<FenViolation>,
) {
    if field == "-" {
        return;
    }

    let &[file @ b'a'..=b'h', rank @ (b'3' | b'6')] = field.as_bytes() else {
        violations.push(FenViolation::InvalidEnPassant);
        return;
    };

    let Some(side_to_move) = side_to_move else {
        return;
    };

    let (target_rank, pawn_rank, origin_rank, pawn) = if side_to_move == Side::WHITE {
        (b'6', 4, 6, 'p')
    } else {
        (b'3', 3, 1, 'P')
    };

    if rank != target_rank {
        violations.push(FenViolation::InvalidEnPassant);
        return;
    }

    let Some(squares) = squares else {
        return;
    };

    let file = usize::from(file - b'a');
    let target_rank = usize::from(rank - b'1');

    if squares[pawn_rank * 8 + file] != pawn
        || squares[target_rank * 8 + file] != EMPTY
        || squares[origin_rank * 8 + file] != EMPTY
    {
        violations.push(FenViolation::ImpossibleEnPassant);
    }
}

fn is_opponent_in_check(squares: &[char; 64], side_to_move: Side) -> bool {
    let mut board = ChessBoard::default();

    for (index, &symbol) in squares.iter().enumerate() {
        if let Some(piece) = piece_from_symbol(symbol) {
            let side = if symbol.is_ascii_uppercase() {
                Side::WHITE
            } else {
                Side::BLACK
            };

            board.set_piece_on_square(Square::from_coords((index / 8) as u8, (index % 8) as u8), piece, side);
        }
    }

    let defender = side_to_move.flipped();
    board.is_square_attacked(board.king_square(defender), defender)
}

fn expand_rank(rank: &str) -> Option<[char; 8]> {
    let mut row = [EMPTY; 8];
    let mut file = 0;

    for symbol in rank.chars() {
        match symbol {
            '1'..='8' => file += symbol as usize - '0' as usize,
            'p' | 'n' | 'b' | 'r' | 'q' | 'k' | 'P' | 'N' | 'B' | 'R' | 'Q' | 'K' => {
                *row.get_mut(file)? = symbol;
                file += 1;
            }
            _ => return None,
        }
    }

    (file == 8).then_some(row)
}

fn piece_from_symbol(symbol: char) -> Option<Piece> {
    match symbol.to_ascii_lowercase() {
        'p' => Some(Piece::PAWN),
        'n' => Some(Piece::KNIGHT),
        'b' => Some(Piece::BISHOP),
        'r' => Some(Piece::ROOK),
        'q' => Some(Piece::QUEEN),
        'k' => Some(Piece::KING),
        _ => None,
    }
}

fn count(squares: &[char; 64], symbol: char) -> usize {
    squares.iter().filter(|&&square| square == symbol).count()
}

fn side_name(side: Side) -> &'static str {
    if side == Side::WHITE {
        "white"
    } else {
        "black"
    }
}
