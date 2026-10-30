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

use valkyrie_chess::Side;
use valkyrie_engine::prelude::*;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoViolation {
    MissingValue(String),
    InvalidValue { argument: String, value: String },
}

impl std::fmt::Display for GoViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingValue(argument) => write!(f, "Missing value for {argument}"),
            Self::InvalidValue { argument, value } => {
                write!(f, "Incorrect value '{value}' for {argument}")
            }
        }
    }
}

pub fn parse_go(args: &[&str], side: Side) -> Result<SearchLimits, Vec<GoViolation>> {
    let mut limits = SearchLimits::default();
    let mut violations = Vec::new();
    let mut idx = 0;

    while idx < args.len() {
        let argument = args[idx];
        idx += 1;

        if argument == "infinite" {
            limits.set_infinite(true);
            continue;
        }

        if !VALUE_ARGUMENTS.contains(&argument) {
            continue;
        }

        let Some(&value) = args.get(idx).filter(|token| !is_argument(token)) else {
            violations.push(GoViolation::MissingValue(argument.to_string()));
            continue;
        };

        idx += 1;

        match value.parse::<i64>() {
            Ok(value) => apply_limit(&mut limits, argument, value.max(0) as u64, side),
            Err(_) => violations.push(GoViolation::InvalidValue {
                argument: argument.to_string(),
                value: value.to_string(),
            }),
        }
    }

    if violations.is_empty() {
        Ok(limits)
    } else {
        Err(violations)
    }
}

fn is_argument(token: &str) -> bool {
    token == "infinite" || VALUE_ARGUMENTS.contains(&token)
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

    fn tokens(command: &str) -> Vec<&str> {
        command.split_whitespace().collect()
    }

    #[test]
    fn reads_the_limits_for_the_side_to_move() {
        let command =
            "wtime 1000 btime 2000 winc 10 binc 20 movestogo 5 movetime 300 depth 7 nodes 99";

        let white = parse_go(&tokens(command), Side::WHITE).unwrap();
        assert_eq!(white.time(), Some(1000));
        assert_eq!(white.increment(), Some(10));

        let black = parse_go(&tokens(command), Side::BLACK).unwrap();
        assert_eq!(black.time(), Some(2000));
        assert_eq!(black.increment(), Some(20));

        assert_eq!(white.moves_to_go(), Some(5));
        assert_eq!(white.move_time(), Some(300));
        assert_eq!(white.depth(), Some(7));
        assert_eq!(white.nodes(), Some(99));
    }

    #[test]
    fn infinite_and_empty_commands_parse() {
        assert!(
            parse_go(&tokens("infinite"), Side::WHITE)
                .unwrap()
                .is_infinite()
        );

        let empty = parse_go(&[], Side::WHITE).unwrap();
        assert!(!empty.is_infinite());
        assert_eq!(empty.time(), None);
        assert_eq!(empty.nodes(), None);
    }

    #[test]
    fn negative_values_clamp_to_zero() {
        let limits = parse_go(&tokens("wtime -500 nodes -1"), Side::WHITE).unwrap();

        assert_eq!(limits.time(), Some(0));
        assert_eq!(limits.nodes(), Some(0));
    }

    #[test]
    fn unknown_tokens_are_ignored() {
        let limits = parse_go(&tokens("ponder searchmoves e2e4 nodes 10"), Side::WHITE).unwrap();

        assert_eq!(limits.nodes(), Some(10));
    }

    #[test]
    fn missing_values_are_reported() {
        let violations = parse_go(&tokens("nodes"), Side::WHITE).unwrap_err();
        assert_eq!(violations, vec![GoViolation::MissingValue("nodes".into())]);

        let violations = parse_go(&tokens("depth nodes 5"), Side::WHITE).unwrap_err();
        assert_eq!(violations, vec![GoViolation::MissingValue("depth".into())]);
    }

    #[test]
    fn invalid_values_are_reported_together() {
        let violations = parse_go(&tokens("depth x nodes 1.5 movetime"), Side::WHITE).unwrap_err();

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
