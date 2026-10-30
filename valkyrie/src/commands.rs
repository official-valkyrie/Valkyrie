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

use std::{sync::Arc, thread::JoinHandle};

use valkyrie_engine::prelude::*;

use crate::interrupt;

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
    pub during_search: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dispatch {
    Done,
    Deferred,
    Unknown,
}

struct Search {
    handle: JoinHandle<(Option<Move>, SearchStats)>,
    infinite: bool,
}

pub struct Shell<C: EngineConfig> {
    engine: Arc<Engine<C>>,
    pub registry: Registry<C>,
    pub shutdown: bool,
    search: Option<Search>,
}

impl<C: EngineConfig> Shell<C> {
    pub fn new(engine: Engine<C>, registry: Registry<C>) -> Self {
        Self {
            engine: Arc::new(engine),
            registry,
            shutdown: false,
            search: None,
        }
    }

    pub fn engine(&self) -> &Engine<C> {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut Engine<C> {
        Arc::get_mut(&mut self.engine).expect("the engine is shared while a search is running")
    }

    pub fn shared_engine(&self) -> &Arc<Engine<C>> {
        &self.engine
    }

    pub fn start_search(
        &mut self,
        handle: JoinHandle<(Option<Move>, SearchStats)>,
        infinite: bool,
    ) {
        interrupt::set_searching(true);
        self.search = Some(Search { handle, infinite });
    }

    pub fn is_searching(&self) -> bool {
        self.search.is_some()
    }

    pub fn is_searching_infinitely(&self) -> bool {
        self.search.as_ref().is_some_and(|search| search.infinite)
    }

    pub fn reap_search(&mut self) {
        if self
            .search
            .as_ref()
            .is_some_and(|search| search.handle.is_finished())
        {
            self.wait_for_search();
        }
    }

    pub fn wait_for_search(&mut self) {
        if let Some(search) = self.search.take() {
            let _ = search.handle.join();
            interrupt::set_searching(false);
        }
    }

    pub fn dispatch(&mut self, line: &str) -> Dispatch {
        let tokens: Vec<&str> = line.split_whitespace().collect();

        let Some(&name) = tokens.first() else {
            return Dispatch::Unknown;
        };

        let Some((run, during_search)) = self
            .registry
            .find(name)
            .map(|command| (command.run, command.during_search))
        else {
            return Dispatch::Unknown;
        };

        if self.is_searching() && !during_search {
            return Dispatch::Deferred;
        }

        run(self, &tokens[1..]);

        Dispatch::Done
    }
}
