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

use valkyrie_engine::prelude::*;

use crate::input::Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Uci,
    Tools,
    General,
}

impl Category {
    pub const ALL: [Self; 3] = [Self::Uci, Self::Tools, Self::General];

    pub const fn title(self) -> &'static str {
        match self {
            Self::Uci => "UCI commands",
            Self::Tools => "Tools",
            Self::General => "General",
        }
    }
}

pub struct Command<C: EngineConfig> {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub category: Category,
    pub cli: bool,
    pub usage: &'static str,
    pub description: &'static str,
    pub run: fn(&mut Shell<C>, &[&str]),
}

pub struct Registry<C: EngineConfig> {
    commands: Vec<Command<C>>,
}

impl<C: EngineConfig> Registry<C> {
    pub fn new(commands: Vec<Command<C>>) -> Self {
        Self { commands }
    }

    pub fn commands(&self) -> &[Command<C>] {
        &self.commands
    }

    pub fn find(&self, token: &str) -> Option<&Command<C>> {
        self.commands
            .iter()
            .find(|command| command.name == token || command.aliases.contains(&token))
    }
}

pub struct Shell<C: EngineConfig> {
    pub engine: Engine<C>,
    pub input: Input,
    pub registry: Registry<C>,
    pub shutdown: bool,
}

impl<C: EngineConfig> Shell<C> {
    pub fn execute(&mut self, tokens: &[&str]) -> bool {
        let Some(&name) = tokens.first() else {
            return false;
        };

        let Some(run) = self.registry.find(name).map(|command| command.run) else {
            return false;
        };

        run(self, &tokens[1..]);

        true
    }
}
