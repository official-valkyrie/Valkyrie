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

use std::cell::RefCell;

use valkyrie_engine::prelude::*;

use crate::{
    cli::FLAGS,
    commands::{Category, Shell},
};

const LINE_WIDTH: usize = 80;
const MIN_DESCRIPTION_WIDTH: usize = 24;

pub fn render<C: EngineConfig>(shell: &Shell<C>) -> String {
    let mut sections: Vec<(&str, Vec<(String, String)>)> = Vec::new();

    sections.push((
        "Flags",
        FLAGS
            .iter()
            .map(|info| (format!("--{}", info.name), info.description.to_string()))
            .collect(),
    ));

    for category in Category::ALL {
        let rows = shell
            .registry
            .commands()
            .iter()
            .filter(|command| command.category == category)
            .map(|command| {
                let aliases = if command.aliases.is_empty() {
                    String::new()
                } else {
                    format!(
                        " ({})",
                        command
                            .aliases
                            .iter()
                            .map(|alias| if command.cli_stage.is_none() {
                                alias.to_string()
                            } else if alias.len() == 1 {
                                format!("-{alias}")
                            } else {
                                format!("--{alias}")
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };

                (
                    command.usage.to_string(),
                    format!(
                        "{}{aliases}{}",
                        command.description,
                        if command.cli_stage.is_some() {
                            ""
                        } else {
                            ", interactive only"
                        }
                    ),
                )
            })
            .collect();

        sections.push((category.title(), rows));
    }

    let specs = RefCell::new(Vec::new());
    shell
        .engine()
        .params()
        .visit_options(&|spec| specs.borrow_mut().push(spec.clone()));
    let specs = specs.into_inner();

    sections.push((
        "Options",
        option_rows(specs.iter().filter(|spec| !spec.tunable)),
    ));
    sections.push((
        "Tunables",
        option_rows(specs.iter().filter(|spec| spec.tunable)),
    ));

    let width = sections
        .iter()
        .flat_map(|(_, rows)| rows)
        .map(|(left, _)| left.len())
        .max()
        .unwrap_or(0);
    let description_width = LINE_WIDTH
        .saturating_sub(width + 4)
        .max(MIN_DESCRIPTION_WIDTH);

    let mut text = format!(
        "{}\n\nUsage: valkyrie [FLAGS] [--option.NAME VALUE]... [COMMAND [ARGS]]...\n       Leading dashes are optional on flags, commands and options.\n       Options run first, then position, then commands in the given order.\n",
        env!("ENGINE_NAME")
    );

    for (title, rows) in sections.iter().filter(|(_, rows)| !rows.is_empty()) {
        text += &format!("\n{title}:\n");

        for (left, right) in rows {
            for (idx, line) in wrap(right, description_width).iter().enumerate() {
                let left = if idx == 0 { left.as_str() } else { "" };
                text += &format!("  {left:<width$}  {line}\n");
            }
        }
    }

    text.trim_end().to_string()
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = vec![String::new()];

    for word in text.split_whitespace() {
        let line = lines.last_mut().unwrap();

        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(word.to_string());
        } else {
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
    }

    lines
}

fn option_rows<'a>(specs: impl Iterator<Item = &'a OptionSpec>) -> Vec<(String, String)> {
    specs
        .map(|spec| {
            let value = match spec.kind {
                OptionKind::Spin => " <INT>",
                OptionKind::Check => " <BOOL>",
                OptionKind::String if spec.range.is_some() => " <NUMBER>",
                OptionKind::String => " <TEXT>",
                OptionKind::Button => "",
            };

            let mut details = match spec.kind {
                OptionKind::Button => "Press the button".to_string(),
                _ if spec.default.is_empty() => "default \"\"".to_string(),
                _ => format!("default {}", spec.default),
            };

            if let Some((min, max)) = &spec.range {
                details += &format!(", range {min}..{max}");
            }

            (format!("--option.{}{value}", spec.name), details)
        })
        .collect()
}
