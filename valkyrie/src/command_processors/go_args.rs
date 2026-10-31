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

use valkyrie_chess::{ChessBoard, Side};
use valkyrie_engine::prelude::*;

use crate::command_processors::position_args::find_legal_moves;

const VALUE_ARGUMENTS: [&str; 8] = [
    "wtime",
    "btime",
    "winc",
    "binc",
    "movestogo",
    "movetime",
    "depth",
    "nodes",
];
const FLAG_ARGUMENTS: [&str; 2] = ["infinite", "searchmoves"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoViolation {
    MissingValue(String),
    InvalidValue { argument: String, value: String },
    IllegalMove(String),
}

impl std::fmt::Display for GoViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingValue(argument) => write!(f, "Missing value for {argument}"),
            Self::InvalidValue { argument, value } => {
                write!(f, "Incorrect value '{value}' for {argument}")
            }
            Self::IllegalMove(mv) => write!(f, "Illegal move: {mv}"),
        }
    }
}

pub fn parse_go(
    args: &[&str],
    board: &ChessBoard,
    chess960: bool,
) -> Result<SearchLimits, Vec<GoViolation>> {
    let mut limits = SearchLimits::default();
    let mut violations = Vec::new();
    let mut limited = false;
    let mut idx = 0;

    while idx < args.len() {
        let argument = args[idx];
        idx += 1;

        match argument {
            "infinite" => limits.set_infinite(true),
            "searchmoves" => {
                let mut search_moves = Vec::new();

                while let Some(&mv) = args.get(idx).filter(|token| !is_argument(token)) {
                    idx += 1;

                    let found = find_legal_moves(board, mv, chess960);
                    if found.is_empty() {
                        violations.push(GoViolation::IllegalMove(mv.to_string()));
                    }

                    search_moves.extend(found);
                }

                limits.set_search_moves(search_moves);
            }
            _ if VALUE_ARGUMENTS.contains(&argument) => {
                limited = true;

                let Some(&value) = args.get(idx).filter(|token| !is_argument(token)) else {
                    violations.push(GoViolation::MissingValue(argument.to_string()));
                    continue;
                };

                idx += 1;

                match value.parse::<i64>() {
                    Ok(value) => {
                        apply_limit(&mut limits, argument, value.max(0) as u64, board.side())
                    }
                    Err(_) => violations.push(GoViolation::InvalidValue {
                        argument: argument.to_string(),
                        value: value.to_string(),
                    }),
                }
            }
            _ => {}
        }
    }

    if !limited {
        limits.set_infinite(true);
    }

    if violations.is_empty() {
        Ok(limits)
    } else {
        Err(violations)
    }
}

fn is_argument(token: &str) -> bool {
    FLAG_ARGUMENTS.contains(&token) || VALUE_ARGUMENTS.contains(&token)
}

fn apply_limit(limits: &mut SearchLimits, argument: &str, value: u64, side: Side) {
    match argument {
        "wtime" if side == Side::WHITE => limits.set_time(Some(value)),
        "btime" if side == Side::BLACK => limits.set_time(Some(value)),
        "winc" if side == Side::WHITE => limits.set_increment(Some(value)),
        "binc" if side == Side::BLACK => limits.set_increment(Some(value)),
        "movestogo" => limits.set_moves_to_go(Some(value)),
        "movetime" => limits.set_move_time(Some(value)),
        "depth" => limits.set_depth(Some(value)),
        "nodes" => limits.set_nodes(Some(value)),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use valkyrie_chess::FEN;

    fn tokens(command: &str) -> Vec<&str> {
        command.split_whitespace().collect()
    }

    fn board(fen: &str) -> ChessBoard {
        ChessBoard::from(&FEN::from(fen))
    }

    fn white() -> ChessBoard {
        ChessBoard::from(&FEN::start_position())
    }

    fn black() -> ChessBoard {
        board("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1")
    }

    fn parse(command: &str) -> Result<SearchLimits, Vec<GoViolation>> {
        parse_go(&tokens(command), &white(), false)
    }

    #[test]
    fn reads_the_limits_for_the_side_to_move() {
        let command =
            "wtime 1000 btime 2000 winc 10 binc 20 movestogo 5 movetime 300 depth 7 nodes 99";

        let white = parse_go(&tokens(command), &white(), false).unwrap();
        assert_eq!(white.time(), Some(1000));
        assert_eq!(white.increment(), Some(10));

        let black = parse_go(&tokens(command), &black(), false).unwrap();
        assert_eq!(black.time(), Some(2000));
        assert_eq!(black.increment(), Some(20));

        assert_eq!(white.moves_to_go(), Some(5));
        assert_eq!(white.move_time(), Some(300));
        assert_eq!(white.depth(), Some(7));
        assert_eq!(white.nodes(), Some(99));
        assert!(!white.is_infinite());
    }

    #[test]
    fn a_go_without_limits_is_infinite() {
        assert!(parse("infinite").unwrap().is_infinite());
        assert!(parse("").unwrap().is_infinite());
        assert!(parse("searchmoves e2e4").unwrap().is_infinite());
        assert!(!parse("btime 1000").unwrap().is_infinite());
    }

    #[test]
    fn negative_values_clamp_to_zero() {
        let limits = parse("wtime -500 nodes -1").unwrap();

        assert_eq!(limits.time(), Some(0));
        assert_eq!(limits.nodes(), Some(0));
    }

    #[test]
    fn unknown_tokens_are_ignored() {
        assert_eq!(parse("foo nodes 10 bar").unwrap().nodes(), Some(10));
    }

    #[test]
    fn search_moves_are_read() {
        let limits = parse("searchmoves e2e4 g1f3 nodes 10").unwrap();

        assert_eq!(limits.nodes(), Some(10));
        assert_eq!(
            limits
                .search_moves()
                .iter()
                .map(|mv| mv.to_string(false))
                .collect::<Vec<_>>(),
            ["e2e4", "g1f3"]
        );
        assert!(limits.allows(limits.search_moves()[0]));
    }

    #[test]
    fn illegal_search_moves_are_reported() {
        assert_eq!(
            parse("searchmoves e2e4 e2e5 a1a8").unwrap_err(),
            vec![
                GoViolation::IllegalMove("e2e5".into()),
                GoViolation::IllegalMove("a1a8".into()),
            ]
        );
    }

    #[test]
    fn missing_values_are_reported() {
        let violations = parse("nodes").unwrap_err();
        assert_eq!(violations, vec![GoViolation::MissingValue("nodes".into())]);

        let violations = parse("depth nodes 5").unwrap_err();
        assert_eq!(violations, vec![GoViolation::MissingValue("depth".into())]);
    }

    #[test]
    fn invalid_values_are_reported_together() {
        let violations = parse("depth x nodes 1.5 movetime").unwrap_err();

        assert_eq!(
            violations,
            vec![
                GoViolation::InvalidValue {
                    argument: "depth".into(),
                    value: "x".into()
                },
                GoViolation::InvalidValue {
                    argument: "nodes".into(),
                    value: "1.5".into()
                },
                GoViolation::MissingValue("movetime".into()),
            ]
        );
    }
}
