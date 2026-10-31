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

use valkyrie_engine::{
    engine::tree::components::{HasPolicy, HasScore, HasVisits},
    prelude::*,
};

use crate::{
    command_processors::{go_args::parse_go, position_args::parse_position, print_violations},
    commands::{Category, Command, Shell},
    execution_queue::stage,
    uci_protocol,
};

pub struct UciProcessor;
impl UciProcessor {
    pub fn commands<C: EngineConfig>() -> Vec<Command<C>>
    where
        C::Search: SearchStrategy<C>,
        C::TimeManager: TimeManagerStrategy<C>,
        C::BestMove: BestMoveStrategy<C>,
        C::Node: HasVisits,
        C::Edge: HasScore + HasVisits + HasPolicy,
    {
        vec![
            Command {
                name: "uci",
                aliases: &[],
                category: Category::Uci,
                cli_stage: Some(stage::RUN),
                during_search: false,
                usage: "uci",
                description: "Print the engine name and options, then uciok",
                run: Self::uci,
            },
            Command {
                name: "isready",
                aliases: &[],
                category: Category::Uci,
                cli_stage: Some(stage::RUN),
                during_search: true,
                usage: "isready",
                description: "Answer readyok",
                run: Self::is_ready,
            },
            Command {
                name: "ucinewgame",
                aliases: &[],
                category: Category::Uci,
                cli_stage: None,
                during_search: false,
                usage: "ucinewgame",
                description: "Clear the tree and set the start position",
                run: Self::new_game,
            },
            Command {
                name: "setoption",
                aliases: &[],
                category: Category::Uci,
                cli_stage: Some(stage::CONFIGURE),
                during_search: false,
                usage: "setoption name <NAME> [value <VALUE>]",
                description: "Set an option or press a button",
                run: Self::set_option,
            },
            Command {
                name: "position",
                aliases: &[],
                category: Category::Uci,
                cli_stage: Some(stage::SETUP),
                during_search: false,
                usage: "position startpos|fen <FEN> [moves <MOVES>...]",
                description: "Set the current position",
                run: Self::position,
            },
            Command {
                name: "go",
                aliases: &[],
                category: Category::Uci,
                cli_stage: Some(stage::RUN),
                during_search: false,
                usage: "go [LIMITS...]",
                description: "Search the current position (wtime btime winc binc movestogo movetime depth nodes searchmoves infinite), infinite without limits",
                run: Self::go,
            },
            Command {
                name: "stop",
                aliases: &["s"],
                category: Category::Uci,
                cli_stage: None,
                during_search: true,
                usage: "stop",
                description: "Stop the running search",
                run: Self::stop,
            },
            Command {
                name: "quit",
                aliases: &["q"],
                category: Category::Uci,
                cli_stage: None,
                during_search: true,
                usage: "quit",
                description: "Exit the engine",
                run: Self::quit,
            },
        ]
    }

    fn uci<C: EngineConfig>(shell: &mut Shell<C>, _args: &[&str]) {
        uci_protocol::identify(env!("ENGINE_NAME"), env!("CARGO_PKG_AUTHORS"));
        shell.engine().params().visit_options(&uci_protocol::option);
        uci_protocol::uci_ok();
    }

    fn is_ready<C: EngineConfig>(_shell: &mut Shell<C>, _args: &[&str]) {
        uci_protocol::ready_ok();
    }

    fn new_game<C: EngineConfig>(shell: &mut Shell<C>, _args: &[&str]) {
        shell.engine_mut().new_game();
    }

    fn stop<C: EngineConfig>(shell: &mut Shell<C>, _args: &[&str]) {
        shell.engine().stop();
    }

    fn quit<C: EngineConfig>(shell: &mut Shell<C>, _args: &[&str]) {
        shell.engine().stop();
        shell.shutdown = true;
    }

    fn position<C: EngineConfig>(shell: &mut Shell<C>, args: &[&str]) {
        let engine = shell.engine_mut();

        match parse_position(args, engine.params().general().chess960()) {
            Ok(position) => {
                engine.set_position(&position);
                engine.view().position_changed(engine.position().board());
            }
            Err(violations) => print_violations(engine, &violations),
        }
    }

    fn set_option<C: EngineConfig>(shell: &mut Shell<C>, args: &[&str]) {
        let engine = shell.engine_mut();

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
            engine.view().option_changed(&name_str, &value_str);
        } else {
            engine.view().button_pressed(&name_str);
        }
    }

    fn go<C: EngineConfig>(shell: &mut Shell<C>, args: &[&str])
    where
        C::Search: SearchStrategy<C>,
        C::TimeManager: TimeManagerStrategy<C>,
        C::BestMove: BestMoveStrategy<C>,
        C::Node: HasVisits,
        C::Edge: HasScore + HasVisits + HasPolicy,
    {
        let engine = shell.engine();

        let limits = match parse_go(
            args,
            engine.position().board(),
            engine.params().general().chess960(),
        ) {
            Ok(limits) => limits,
            Err(violations) => {
                print_violations(engine, &violations);
                return;
            }
        };

        let infinite = limits.is_infinite();
        let handle = Engine::spawn_search(shell.shared_engine(), limits);

        shell.start_search(handle, infinite);
    }
}
