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

use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    Simple,
    Minimal,
    Interactive,
}

pub struct FlagInfo {
    pub flag: Flag,
    pub name: &'static str,
    pub description: &'static str,
}

pub const FLAGS: &[FlagInfo] = &[
    FlagInfo {
        flag: Flag::Simple,
        name: "simple",
        description: "Plain text output (default)",
    },
    FlagInfo {
        flag: Flag::Minimal,
        name: "minimal",
        description: "Only what a UCI GUI needs",
    },
    FlagInfo {
        flag: Flag::Interactive,
        name: "interactive",
        description: "Keep reading commands after running the ones given",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayChoice {
    #[default]
    Simple,
    Minimal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    SetOption { name: String, value: String },
    Command(Vec<String>),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Cli {
    pub display: DisplayChoice,
    pub interactive: bool,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliViolation {
    UnknownFlag(String),
    UnexpectedArgument(String),
    ConflictingDisplays,
}

impl Display for CliViolation {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::UnknownFlag(flag) => write!(f, "Unknown flag '{flag}', try --help"),
            Self::UnexpectedArgument(argument) => write!(
                f,
                "Unexpected argument '{argument}', expected a command or flag, try --help"
            ),
            Self::ConflictingDisplays => write!(f, "Flags --simple and --minimal conflict"),
        }
    }
}

pub struct Lookup {
    pub name: &'static str,
    pub primary: bool,
}

enum Token<'a> {
    Long(&'a str),
    Short(&'a str),
    Bare(&'a str),
}

fn classify(token: &str) -> Token<'_> {
    if let Some(body) = token.strip_prefix("--").filter(|body| !body.is_empty()) {
        return Token::Long(body);
    }

    if let Some(body) = token
        .strip_prefix('-')
        .filter(|body| body.len() == 1 && body.chars().all(|symbol| symbol.is_ascii_alphabetic()))
    {
        return Token::Short(body);
    }

    Token::Bare(token)
}

fn is_boundary(token: &str, lookup: &dyn Fn(&str) -> Option<Lookup>) -> bool {
    match classify(token) {
        Token::Long(_) | Token::Short(_) => true,
        Token::Bare(body) => {
            FLAGS.iter().any(|info| info.name == body)
                || body.starts_with("option.")
                || lookup(body).is_some_and(|found| found.primary)
        }
    }
}

pub fn parse(
    args: &[String],
    lookup: &dyn Fn(&str) -> Option<Lookup>,
) -> std::result::Result<Cli, Vec<CliViolation>> {
    let mut cli = Cli::default();
    let mut violations = Vec::new();
    let mut chosen_display = None;
    let mut idx = 0;

    while idx < args.len() {
        let token = &args[idx];
        idx += 1;

        let (body, dashes) = match classify(token) {
            Token::Long(body) => (body, 2),
            Token::Short(body) => (body, 1),
            Token::Bare(body) => (body, 0),
        };
        let dashed = dashes > 0;

        if let Some((category, rest)) = body.split_once('.').filter(|_| dashes != 1) {
            if category != "option" {
                violations.push(if dashed {
                    CliViolation::UnknownFlag(token.clone())
                } else {
                    CliViolation::UnexpectedArgument(token.clone())
                });
                continue;
            }

            let (name, value) = match rest.split_once('=') {
                Some((name, value)) => (name.to_string(), value.to_string()),
                None => {
                    let value = match args.get(idx) {
                        Some(next) if !is_boundary(next, lookup) => {
                            idx += 1;
                            next.clone()
                        }
                        _ => String::new(),
                    };

                    (rest.to_string(), value)
                }
            };

            cli.items.push(Item::SetOption { name, value });
            continue;
        }

        if let Some(info) = FLAGS
            .iter()
            .find(|info| info.name == body)
            .filter(|_| dashes != 1)
        {
            let choice = match info.flag {
                Flag::Simple => Some(DisplayChoice::Simple),
                Flag::Minimal => Some(DisplayChoice::Minimal),
                Flag::Interactive => {
                    cli.interactive = true;
                    None
                }
            };

            if let Some(choice) = choice {
                if chosen_display.is_some_and(|previous| previous != choice) {
                    violations.push(CliViolation::ConflictingDisplays);
                }
                chosen_display = Some(choice);
            }

            continue;
        }

        if let Some(found) = lookup(body).filter(|found| match dashes {
            0 => found.primary,
            1 => body.len() == 1,
            _ => body.len() > 1,
        }) {
            let mut tokens = vec![found.name.to_string()];

            while idx < args.len() && !is_boundary(&args[idx], lookup) {
                tokens.push(args[idx].clone());
                idx += 1;
            }

            cli.items.push(Item::Command(tokens));
            continue;
        }

        violations.push(if dashed {
            CliViolation::UnknownFlag(token.clone())
        } else {
            CliViolation::UnexpectedArgument(token.clone())
        });
    }

    cli.display = chosen_display.unwrap_or_default();

    if violations.is_empty() {
        Ok(cli)
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(token: &str) -> Option<Lookup> {
        const COMMANDS: [(&str, &[&str]); 7] = [
            ("go", &[]),
            ("position", &[]),
            ("perft", &[]),
            ("bench", &[]),
            ("help", &["h"]),
            ("draw", &["d"]),
            ("quit", &["q"]),
        ];

        COMMANDS
            .iter()
            .find(|(name, aliases)| *name == token || aliases.contains(&token))
            .map(|(name, _)| Lookup {
                name,
                primary: *name == token,
            })
    }

    fn parse_args(command: &str) -> std::result::Result<Cli, Vec<CliViolation>> {
        let args: Vec<String> = command.split_whitespace().map(str::to_string).collect();

        parse(&args, &lookup)
    }

    fn command(tokens: &[&str]) -> Item {
        Item::Command(tokens.iter().map(|token| token.to_string()).collect())
    }

    fn option(name: &str, value: &str) -> Item {
        Item::SetOption {
            name: name.into(),
            value: value.into(),
        }
    }

    #[test]
    fn no_arguments_start_the_interactive_loop() {
        assert_eq!(parse_args("").unwrap(), Cli::default());
    }

    #[test]
    fn options_flags_and_commands_mix() {
        let cli = parse_args("--minimal --option.Hash 256 go nodes 1000").unwrap();

        assert_eq!(cli.display, DisplayChoice::Minimal);
        assert_eq!(
            cli.items,
            vec![option("Hash", "256"), command(&["go", "nodes", "1000"])]
        );
    }

    #[test]
    fn dashes_are_optional_on_commands_flags_and_options() {
        let dashed = parse_args("--minimal --option.Hash 256 --bench").unwrap();
        let bare = parse_args("minimal option.Hash 256 bench").unwrap();

        assert_eq!(dashed, bare);
    }

    #[test]
    fn options_accept_equals_and_negative_values() {
        let cli = parse_args("--option.cpuct=1.5 option.Other -1.5").unwrap();

        assert_eq!(
            cli.items,
            vec![option("cpuct", "1.5"), option("Other", "-1.5")]
        );
    }

    #[test]
    fn buttons_have_no_value() {
        let cli = parse_args("--option.ClearHash bench").unwrap();

        assert_eq!(
            cli.items,
            vec![option("ClearHash", ""), command(&["bench"])]
        );
    }

    #[test]
    fn commands_end_at_the_next_item() {
        let cli =
            parse_args("position fen 8/8/8/8/8/8/8/8 w - - 0 1 moves e2e4 go depth 3").unwrap();

        assert_eq!(
            cli.items,
            vec![
                command(&[
                    "position",
                    "fen",
                    "8/8/8/8/8/8/8/8",
                    "w",
                    "-",
                    "-",
                    "0",
                    "1",
                    "moves",
                    "e2e4"
                ]),
                command(&["go", "depth", "3"]),
            ]
        );
    }

    #[test]
    fn single_letter_aliases_take_one_dash() {
        assert_eq!(parse_args("-h").unwrap().items, vec![command(&["help"])]);
        assert_eq!(parse_args("-d").unwrap().items, vec![command(&["draw"])]);
        assert_eq!(
            parse_args("--draw").unwrap().items,
            vec![command(&["draw"])]
        );
        assert_eq!(
            parse_args("h").unwrap_err(),
            vec![CliViolation::UnexpectedArgument("h".into())]
        );
        assert_eq!(
            parse_args("--d -draw").unwrap_err(),
            vec![
                CliViolation::UnknownFlag("--d".into()),
                CliViolation::UnexpectedArgument("-draw".into()),
            ]
        );
    }

    #[test]
    fn interactive_flag_is_remembered() {
        let cli = parse_args("bench interactive").unwrap();

        assert!(cli.interactive);
        assert_eq!(cli.items, vec![command(&["bench"])]);
    }

    #[test]
    fn unknown_input_is_reported_together() {
        assert_eq!(
            parse_args("--bogus --foo.bar stray").unwrap_err(),
            vec![
                CliViolation::UnknownFlag("--bogus".into()),
                CliViolation::UnknownFlag("--foo.bar".into()),
                CliViolation::UnexpectedArgument("stray".into()),
            ]
        );
    }

    #[test]
    fn conflicting_displays_are_reported() {
        assert_eq!(
            parse_args("--simple --minimal").unwrap_err(),
            vec![CliViolation::ConflictingDisplays]
        );
        assert!(parse_args("--minimal minimal").is_ok());
    }
}
