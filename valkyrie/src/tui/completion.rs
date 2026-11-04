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

use valkyrie_engine::prelude::{OptionKind, OptionSpec};

const MAX_SUGGESTIONS: usize = 8;
const FEN_FIELDS: usize = 6;

const POSITIONS: &[(&str, &str)] = &[
    (
        "Start position",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ),
    (
        "Kiwipete",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    ),
    ("Rook endgame", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"),
    (
        "Promotion tactics",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    ),
];

const GO_KEYWORDS: &[(&str, &str, &[&str])] = &[
    ("depth", "Search to a fixed depth", &["8", "12", "16", "20"]),
    (
        "nodes",
        "Stop after this many nodes",
        &["10000", "100000", "1000000"],
    ),
    (
        "movetime",
        "Think for this many milliseconds",
        &["1000", "5000", "10000", "30000"],
    ),
    (
        "wtime",
        "White clock in milliseconds",
        &["60000", "180000", "300000"],
    ),
    (
        "btime",
        "Black clock in milliseconds",
        &["60000", "180000", "300000"],
    ),
    (
        "winc",
        "White increment in milliseconds",
        &["0", "1000", "2000"],
    ),
    (
        "binc",
        "Black increment in milliseconds",
        &["0", "1000", "2000"],
    ),
    (
        "movestogo",
        "Moves until the next time control",
        &["20", "40"],
    ),
    ("searchmoves", "Restrict the search to these moves", &[]),
    ("infinite", "Search until stopped", &[]),
];

const DEPTH_COMMANDS: &[&str] = &["perft", "perft_no_bulk", "bench"];

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub name: &'static str,
    pub aliases: Vec<&'static str>,
    pub usage: &'static str,
    pub description: &'static str,
    pub category: &'static str,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub engine_name: String,
    pub commands: Vec<CommandInfo>,
    pub options: Vec<OptionSpec>,
}

impl Catalog {
    pub fn command(&self, token: &str) -> Option<&CommandInfo> {
        self.commands.iter().find(|command| {
            command.name.eq_ignore_ascii_case(token)
                || command
                    .aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(token))
        })
    }

    pub fn option(&self, name: &str) -> Option<&OptionSpec> {
        self.options
            .iter()
            .find(|spec| spec.name.eq_ignore_ascii_case(name))
    }
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub insert: String,
    pub label: String,
    pub detail: String,
}

#[derive(Debug, Default)]
pub struct Suggestions {
    pub start: usize,
    pub items: Vec<Suggestion>,
    pub usage: Option<&'static str>,
}

impl Suggestions {
    pub fn ghost(&self, typed: &str, selected: usize) -> Option<String> {
        let item = self.items.get(selected)?;
        let typed_length = typed.chars().count();

        (typed_length > 0
            && item
                .insert
                .to_lowercase()
                .starts_with(&typed.to_lowercase()))
        .then(|| item.insert.chars().skip(typed_length).collect())
    }
}

struct Token<'a> {
    start: usize,
    text: &'a str,
}

fn tokenize(text: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut begin: Option<(usize, usize)> = None;

    for (column, (byte, symbol)) in text.char_indices().enumerate() {
        match (symbol.is_whitespace(), begin) {
            (true, Some((start, from))) => {
                tokens.push(Token {
                    start,
                    text: &text[from..byte],
                });
                begin = None;
            }
            (false, None) => begin = Some((column, byte)),
            _ => {}
        }
    }

    if let Some((start, from)) = begin {
        tokens.push(Token {
            start,
            text: &text[from..],
        });
    }

    tokens
}

fn rank(candidate: &str, typed: &str) -> Option<u8> {
    let candidate = candidate.to_lowercase();
    let typed = typed.to_lowercase();

    if candidate.starts_with(&typed) {
        Some(0)
    } else if typed.chars().count() >= 2 && candidate.contains(&typed) {
        Some(1)
    } else {
        None
    }
}

fn collect(
    candidates: impl Iterator<Item = Suggestion>,
    typed: &str,
    key: impl Fn(&Suggestion) -> String,
) -> Vec<Suggestion> {
    let mut ranked: Vec<(u8, Suggestion)> = candidates
        .filter_map(|candidate| rank(&key(&candidate), typed).map(|rank| (rank, candidate)))
        .collect();

    ranked.sort_by_key(|(rank, _)| *rank);
    ranked
        .into_iter()
        .map(|(_, candidate)| candidate)
        .take(MAX_SUGGESTIONS)
        .collect()
}

fn word(text: &str, detail: &str) -> Suggestion {
    Suggestion {
        insert: format!("{text} "),
        label: text.to_string(),
        detail: detail.to_string(),
    }
}

pub fn suggest(catalog: &Catalog, text: &str) -> Suggestions {
    let tokens = tokenize(text);
    let length = text.chars().count();
    let typing = !text.ends_with(char::is_whitespace);

    let (done, partial) = match tokens.split_last() {
        Some((last, rest)) if typing => (rest, Some(last)),
        _ => (&tokens[..], None),
    };

    let typed = partial.map_or("", |token| token.text);
    let start = partial.map_or(length, |token| token.start);

    let Some(first) = done.first() else {
        if typed.is_empty() {
            return Suggestions::default();
        }

        let items = collect(
            catalog.commands.iter().map(|command| Suggestion {
                insert: format!("{} ", command.name),
                label: command.name.to_string(),
                detail: command.description.to_string(),
            }),
            typed,
            |suggestion| suggestion.label.clone(),
        );

        return Suggestions {
            start,
            usage: items
                .first()
                .and_then(|item| catalog.command(&item.label))
                .map(|command| command.usage),
            items,
        };
    };

    let Some(command) = catalog.command(first.text) else {
        return Suggestions {
            start,
            ..Suggestions::default()
        };
    };

    let arguments = &done[1..];

    let (start, items) = match command.name {
        "setoption" => setoption(catalog, arguments, typed, start),
        "position" => position(arguments, typed, start),
        "go" => go(arguments, typed, start),
        name if DEPTH_COMMANDS.contains(&name) && arguments.is_empty() => (
            start,
            collect(
                ["4", "5", "6", "7"].into_iter().map(|depth| Suggestion {
                    insert: depth.to_string(),
                    label: depth.to_string(),
                    detail: "Depth".to_string(),
                }),
                typed,
                |suggestion| suggestion.label.clone(),
            ),
        ),
        _ => (start, Vec::new()),
    };

    Suggestions {
        start,
        items,
        usage: Some(command.usage),
    }
}

fn setoption(
    catalog: &Catalog,
    arguments: &[Token<'_>],
    typed: &str,
    start: usize,
) -> (usize, Vec<Suggestion>) {
    let Some(first) = arguments.first() else {
        return (
            start,
            collect(
                std::iter::once(word("name", "Choose the option to change")),
                typed,
                |suggestion| suggestion.label.clone(),
            ),
        );
    };

    if first.text != "name" {
        return (start, Vec::new());
    }

    let value_at = arguments.iter().position(|token| token.text == "value");
    let name_tokens = &arguments[1..value_at.unwrap_or(arguments.len())];
    let name = name_tokens
        .iter()
        .map(|token| token.text)
        .collect::<Vec<_>>()
        .join(" ");

    if value_at.is_some() {
        let Some(spec) = catalog.option(&name) else {
            return (start, Vec::new());
        };

        return (start, option_values(spec, typed));
    }

    if let Some(spec) = catalog.option(&name)
        && spec.kind != OptionKind::Button
        && rank("value", typed).is_some()
        && (typed.is_empty() || "value".starts_with(&typed.to_lowercase()))
    {
        return (start, vec![word("value", "Enter the new value")]);
    }

    let name_start = name_tokens.first().map_or(start, |token| token.start);
    let wanted = match (name.is_empty(), typed.is_empty()) {
        (true, _) => typed.to_string(),
        (false, true) => format!("{name} "),
        (false, false) => format!("{name} {typed}"),
    };

    let mut options: Vec<&OptionSpec> = catalog.options.iter().collect();
    options.sort_by_key(|spec| spec.tunable);

    (
        name_start,
        collect(
            options.into_iter().map(|spec| Suggestion {
                insert: if spec.kind == OptionKind::Button {
                    spec.name.clone()
                } else {
                    format!("{} value ", spec.name)
                },
                label: spec.name.clone(),
                detail: describe(spec),
            }),
            &wanted,
            |suggestion| suggestion.label.clone(),
        ),
    )
}

fn describe(spec: &OptionSpec) -> String {
    let mut text = spec.kind.uci_name().to_string();

    if let Some((min, max)) = &spec.range {
        text += &format!(" {min}..{max}");
    }

    if spec.kind != OptionKind::Button && !spec.default.is_empty() {
        text += &format!(" · default {}", spec.default);
    }

    text
}

fn option_values(spec: &OptionSpec, typed: &str) -> Vec<Suggestion> {
    let mut values: Vec<(String, String)> = match spec.kind {
        OptionKind::Check => vec![
            ("true".to_string(), String::new()),
            ("false".to_string(), String::new()),
        ],
        OptionKind::Combo => spec
            .variants
            .iter()
            .map(|variant| (variant.to_string(), String::new()))
            .collect(),
        OptionKind::Spin | OptionKind::String if !spec.default.is_empty() => {
            vec![(spec.default.clone(), describe(spec))]
        }
        _ => Vec::new(),
    };

    for (value, detail) in &mut values {
        if *value == spec.default && detail.is_empty() {
            *detail = "default".to_string();
        }
    }

    collect(
        values.into_iter().map(|(value, detail)| Suggestion {
            insert: value.clone(),
            label: value,
            detail,
        }),
        typed,
        |suggestion| suggestion.label.clone(),
    )
}

fn position(arguments: &[Token<'_>], typed: &str, start: usize) -> (usize, Vec<Suggestion>) {
    let Some(first) = arguments.first() else {
        return (
            start,
            collect(
                [
                    word("startpos", "The standard starting position"),
                    word("fen", "Set up a position from a FEN string"),
                ]
                .into_iter(),
                typed,
                |suggestion| suggestion.label.clone(),
            ),
        );
    };

    let moves = || {
        collect(
            std::iter::once(word("moves", "Play moves from the position")),
            typed,
            |suggestion| suggestion.label.clone(),
        )
    };

    match first.text {
        "startpos" if arguments.len() == 1 => (start, moves()),
        "fen" if !arguments.iter().any(|token| token.text == "moves") => {
            let fields = &arguments[1..];

            if fields.len() >= FEN_FIELDS {
                return (start, moves());
            }

            let fen_start = fields.first().map_or(start, |token| token.start);
            let written = fields
                .iter()
                .map(|token| token.text)
                .collect::<Vec<_>>()
                .join(" ");
            let wanted = match (written.is_empty(), typed.is_empty()) {
                (true, _) => typed.to_string(),
                (false, true) => format!("{written} "),
                (false, false) => format!("{written} {typed}"),
            };

            (
                fen_start,
                collect(
                    POSITIONS.iter().map(|(name, fen)| Suggestion {
                        insert: format!("{fen} "),
                        label: name.to_string(),
                        detail: fen.to_string(),
                    }),
                    "",
                    |_| String::new(),
                )
                .into_iter()
                .filter(|suggestion| {
                    suggestion
                        .detail
                        .to_lowercase()
                        .starts_with(&wanted.to_lowercase())
                })
                .collect(),
            )
        }
        _ => (start, Vec::new()),
    }
}

fn go(arguments: &[Token<'_>], typed: &str, start: usize) -> (usize, Vec<Suggestion>) {
    if let Some(last) = arguments.last()
        && let Some((_, _, values)) = GO_KEYWORDS
            .iter()
            .find(|(keyword, _, _)| *keyword == last.text)
        && !values.is_empty()
    {
        return (
            start,
            collect(
                values.iter().map(|value| Suggestion {
                    insert: format!("{value} "),
                    label: value.to_string(),
                    detail: String::new(),
                }),
                typed,
                |suggestion| suggestion.label.clone(),
            ),
        );
    }

    (
        start,
        collect(
            GO_KEYWORDS
                .iter()
                .filter(|(keyword, _, _)| !arguments.iter().any(|token| token.text == *keyword))
                .map(|(keyword, detail, _)| word(keyword, detail)),
            typed,
            |suggestion| suggestion.label.clone(),
        ),
    )
}
