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

use valkyrie_chess::{ChessBoard, ChessPosition, FEN, FenViolation, Move};

#[derive(Debug, Clone, PartialEq)]
pub enum PositionViolation {
    MissingSetup,
    UnknownSetup(String),
    UnexpectedArgument(String),
    Fen(FenViolation),
    IllegalMove(String),
}

impl std::fmt::Display for PositionViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSetup => write!(f, "Missing position, expected 'startpos' or 'fen <fen>'"),
            Self::UnknownSetup(setup) => {
                write!(
                    f,
                    "Unknown position '{setup}', expected 'startpos' or 'fen <fen>'"
                )
            }
            Self::UnexpectedArgument(argument) => {
                write!(
                    f,
                    "Unexpected argument '{argument}', moves must follow 'moves'"
                )
            }
            Self::Fen(violation) => write!(f, "Invalid FEN: {violation}"),
            Self::IllegalMove(mv) => write!(f, "Illegal move: {mv}"),
        }
    }
}

pub fn parse_position(
    args: &[&str],
    chess960: bool,
) -> Result<ChessPosition, Vec<PositionViolation>> {
    let mut violations = Vec::new();

    let moves_idx = args
        .iter()
        .position(|&arg| arg == "moves")
        .unwrap_or(args.len());
    let moves = args.get(moves_idx + 1..).unwrap_or_default();

    let fen = match &args[..moves_idx] {
        ["startpos", rest @ ..] => {
            violations.extend(
                rest.iter()
                    .map(|arg| PositionViolation::UnexpectedArgument(arg.to_string())),
            );
            FEN::start_position().to_string()
        }
        ["fen", fen @ ..] => fen.join(" "),
        [setup, ..] => return Err(vec![PositionViolation::UnknownSetup(setup.to_string())]),
        [] => return Err(vec![PositionViolation::MissingSetup]),
    };

    let fen_violations = FEN::validate_fen(&fen);
    if !fen_violations.is_empty() {
        violations.extend(fen_violations.into_iter().map(PositionViolation::Fen));
        return Err(violations);
    }

    let mut position = ChessPosition::from(ChessBoard::from(&FEN::from(fen)));

    for &mv_str in moves {
        match find_legal_move(position.board(), mv_str, chess960) {
            Some(mv) => position.make_move_no_mask(mv),
            None => {
                violations.push(PositionViolation::IllegalMove(mv_str.to_string()));
                break;
            }
        }
    }

    if violations.is_empty() {
        Ok(position)
    } else {
        Err(violations)
    }
}

fn find_legal_move(board: &ChessBoard, mv_str: &str, chess960: bool) -> Option<Move> {
    let mut found = None;

    board.map_legal_moves(|mv| {
        if mv.to_string(chess960) == mv_str {
            found = Some(mv);
        }
    });

    found
}
