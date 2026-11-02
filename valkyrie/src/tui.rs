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

pub mod animation;
pub mod canvas;
pub mod completion;
pub mod editor;
pub mod events;
pub mod logo;
pub mod palette;
pub mod renderer;
pub mod screens;
pub mod state;
pub mod theme;
pub mod widgets;

use std::cell::RefCell;

use valkyrie_engine::prelude::*;

use crate::{
    commands::Registry,
    tui::completion::{Catalog, CommandInfo},
};

pub const HELP_COMMAND: &str = "help";

const HELP_SHORTCUT: &str = "?";
const HIDDEN_COMMANDS: [&str; 4] = ["uci", "isready", "clear", "version"];

pub fn catalog<C: EngineConfig>(engine: &Engine<C>, registry: &Registry<C>) -> Catalog {
    let options = RefCell::new(Vec::new());
    engine
        .params()
        .visit_options(&|spec| options.borrow_mut().push(spec.clone()));

    Catalog {
        engine_name: env!("ENGINE_NAME").to_string(),
        commands: registry
            .commands()
            .iter()
            .filter(|command| !HIDDEN_COMMANDS.contains(&command.name))
            .map(|command| {
                let mut aliases = command.aliases.to_vec();

                if command.name == HELP_COMMAND {
                    aliases.push(HELP_SHORTCUT);
                }

                CommandInfo {
                    name: command.name,
                    aliases,
                    usage: command.usage,
                    description: command.description,
                    category: command.category.title(),
                }
            })
            .collect(),
        options: options.into_inner(),
    }
}
