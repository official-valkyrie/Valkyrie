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
pub enum ViewChoice {
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
    pub view: ViewChoice,
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
            FLAGS
                .iter()
                .any(|info| info.name.eq_ignore_ascii_case(body))
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
    let mut chosen_view = None;
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
            .find(|info| info.name.eq_ignore_ascii_case(body))
            .filter(|_| dashes != 1)
        {
            let choice = match info.flag {
                Flag::Simple => Some(ViewChoice::Simple),
                Flag::Minimal => Some(ViewChoice::Minimal),
                Flag::Interactive => {
                    cli.interactive = true;
                    None
                }
            };

            if let Some(choice) = choice {
                if chosen_view.is_some_and(|previous| previous != choice) {
                    violations.push(CliViolation::ConflictingDisplays);
                }
                chosen_view = Some(choice);
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

    cli.view = chosen_view.unwrap_or_default();

    if violations.is_empty() {
        Ok(cli)
    } else {
        Err(violations)
    }
}
