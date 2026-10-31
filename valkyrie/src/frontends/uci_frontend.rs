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

use std::time::Duration;

use valkyrie_engine::prelude::*;

use crate::{
    cli::{Cli, Item},
    commands::{Dispatch, Registry, Shell},
    execution_queue::{ExecutionQueue, stage},
    input::{Input, Poll},
    interrupt,
};

const POLL_INTERVAL: Duration = Duration::from_millis(1);

pub struct UciFrontend<C: EngineConfig> {
    shell: Shell<C>,
    input: Input,
    exit_when_idle: bool,
}

impl<C: EngineConfig> UciFrontend<C> {
    pub fn new(engine: Engine<C>, registry: Registry<C>) -> Self {
        Self {
            shell: Shell::new(engine, registry),
            input: Input::spawn(),
            exit_when_idle: false,
        }
    }

    pub fn run(mut self, cli: &Cli) {
        interrupt::install();
        self.shell.engine().view().start();

        let mut queue = ExecutionQueue::default();

        for item in &cli.items {
            let (stage, line) = self.cli_line(item);
            queue.push(stage, line);
        }

        let ran_command = cli
            .items
            .iter()
            .any(|item| matches!(item, Item::Command(_)));

        for line in queue {
            self.input.push_back(line);
        }

        self.exit_when_idle = ran_command && !cli.interactive;
        self.run_loop();
        self.shell.wait_until_idle();

        self.shell.engine().view().finish();
    }

    fn cli_line(&self, item: &Item) -> (u16, String) {
        match item {
            Item::SetOption { name, value } if value.is_empty() => {
                (stage::CONFIGURE, format!("setoption name {name}"))
            }
            Item::SetOption { name, value } => (
                stage::CONFIGURE,
                format!("setoption name {name} value {value}"),
            ),
            Item::Command(tokens) => {
                let stage = tokens
                    .first()
                    .and_then(|name| self.shell.registry.find(name))
                    .and_then(|command| command.cli_stage)
                    .unwrap_or(stage::RUN);

                (stage, tokens.join(" "))
            }
        }
    }

    fn run_loop(&mut self) {
        loop {
            self.shell.reap();

            let busy = self.shell.is_busy();

            if !busy && (self.shell.shutdown || (self.exit_when_idle && !self.input.has_queued())) {
                break;
            }

            if busy && interrupt::take_stop_request() {
                self.shell.engine().stop();
            }

            let line = if busy {
                match self.input.poll(POLL_INTERVAL) {
                    Poll::Line(line) => line,
                    Poll::Idle => continue,
                    Poll::Closed => {
                        if self.shell.is_searching_infinitely() {
                            self.shell.engine().stop();
                        }
                        continue;
                    }
                }
            } else {
                match self.input.next() {
                    Some(line) => line,
                    None => break,
                }
            };

            if self.shell.dispatch(&line) == Dispatch::Deferred {
                self.input.push_back(line);
            }
        }
    }
}
