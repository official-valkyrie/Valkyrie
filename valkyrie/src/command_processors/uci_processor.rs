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
    command_processors::{go_args::parse_go, position_args::parse_position, print_violations},
    commands::{Category, Command, Shell},
    input::Poll,
};

const POLL_INTERVAL: Duration = Duration::from_millis(1);

pub struct UciProcessor;
impl UciProcessor {
    pub fn commands<C: EngineConfig>() -> Vec<Command<C>>
    where
        C::Search: SearchStrategy<C>,
        C::TimeManager: TimeManagerStrategy<C>,
        C::BestMove: BestMoveStrategy<C>,
    {
        vec![
            Command {
                name: "uci",
                aliases: &[],
                category: Category::Uci,
                cli: true,
                usage: "uci",
                description: "Print the engine name and options, then uciok",
                run: Self::uci,
            },
            Command {
                name: "isready",
                aliases: &[],
                category: Category::Uci,
                cli: true,
                usage: "isready",
                description: "Answer readyok",
                run: Self::is_ready,
            },
            Command {
                name: "setoption",
                aliases: &[],
                category: Category::Uci,
                cli: true,
                usage: "setoption name <NAME> [value <VALUE>]",
                description: "Set an option or press a button",
                run: Self::set_option,
            },
            Command {
                name: "position",
                aliases: &[],
                category: Category::Uci,
                cli: true,
                usage: "position startpos|fen <FEN> [moves <MOVES>...]",
                description: "Set the current position",
                run: Self::position,
            },
            Command {
                name: "go",
                aliases: &[],
                category: Category::Uci,
                cli: true,
                usage: "go [LIMITS...]",
                description: "Search the current position (wtime btime winc binc movestogo movetime depth nodes infinite)",
                run: Self::go,
            },
            Command {
                name: "quit",
                aliases: &["q"],
                category: Category::Uci,
                cli: false,
                usage: "quit",
                description: "Exit the engine",
                run: Self::quit,
            },
        ]
    }

    fn uci<C: EngineConfig>(shell: &mut Shell<C>, _args: &[&str]) {
        let engine = &shell.engine;

        engine
            .display()
            .identify(env!("ENGINE_NAME"), env!("CARGO_PKG_AUTHORS"));
        engine
            .params()
            .visit_options(&|spec| engine.display().option(spec));
        engine.display().handshake_done();
    }

    fn is_ready<C: EngineConfig>(shell: &mut Shell<C>, _args: &[&str]) {
        shell.engine.display().ready();
    }

    fn quit<C: EngineConfig>(shell: &mut Shell<C>, _args: &[&str]) {
        shell.shutdown = true;
    }

    fn position<C: EngineConfig>(shell: &mut Shell<C>, args: &[&str]) {
        let engine = &mut shell.engine;

        match parse_position(args, engine.params().general().chess960()) {
            Ok(position) => {
                engine.set_position(&position);
                engine.print("Position set successfully");
            }
            Err(violations) => print_violations(engine, &violations),
        }
    }

    fn set_option<C: EngineConfig>(shell: &mut Shell<C>, args: &[&str]) {
        let engine = &mut shell.engine;

        let args = match args {
            ["name", rest @ ..] => rest,
            _ => &[],
        };

        let mut name = Vec::new();
        let mut value = Vec::new();
        let mut is_value = false;

        for &token in args {
            if token == "value" {
                is_value = true;
                continue;
            }

            if is_value {
                value.push(token);
            } else {
                name.push(token);
            }
        }

        let name_str = name.join(" ");
        let value_str = value.join(" ");

        if let Err(violation) = engine.set_option(&name_str, &value_str) {
            print_violations(engine, &[violation]);
            return;
        }

        if is_value {
            engine.print(&format!("Option {name_str} has been set to {value_str}"));
        } else {
            engine.print(&format!("Option {name_str} has been triggered"));
        }
    }

    fn go<C: EngineConfig>(shell: &mut Shell<C>, args: &[&str])
    where
        C::Search: SearchStrategy<C>,
        C::TimeManager: TimeManagerStrategy<C>,
        C::BestMove: BestMoveStrategy<C>,
    {
        let Shell {
            engine,
            input,
            shutdown,
            ..
        } = shell;
        let engine = &*engine;

        let limits = match parse_go(args, engine.position().board().side()) {
            Ok(limits) => limits,
            Err(violations) => {
                print_violations(engine, &violations);
                return;
            }
        };

        std::thread::scope(|s| {
            let search = engine.spawn_search(s, &limits);

            while !search.is_finished() {
                match input.poll(POLL_INTERVAL) {
                    Poll::Line(cmd) => match cmd.trim() {
                        "isready" => engine.display().ready(),
                        "stop" | "s" => {
                            engine.stop();
                            break;
                        }
                        "quit" | "q" => {
                            engine.stop();
                            *shutdown = true;
                            break;
                        }
                        _ => input.push_back(cmd),
                    },
                    Poll::Idle => {}
                    Poll::Closed => {
                        if limits.is_infinite() {
                            engine.stop();
                        }
                    }
                }
            }
        });
    }
}
