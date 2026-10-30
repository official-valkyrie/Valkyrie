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

mod cli;
mod command_processors;
mod commands;
mod displays;
mod help;
mod input;

use cli::{DisplayChoice, Item, Lookup};
use command_processors::print_violations;
use commands::Shell;
use displays::{minimal_display::MinimalDisplay, simple_display::SimpleDisplay};
use input::Input;
use valkyrie_engine::engine::builder::backpropagation_strategy::BasicScalarBackpropagate;
use valkyrie_engine::engine::builder::best_move_strategy::MaxQ;
use valkyrie_engine::engine::builder::expansion_strategy::ClassicExpansion;
use valkyrie_engine::engine::builder::exploration_strategy::Puct;
use valkyrie_engine::engine::builder::search_strategy::BasicSequentialMCTS;
use valkyrie_engine::engine::builder::simulation_strategy::MaterialEvaluation;
use valkyrie_engine::engine::builder::time_manager_strategy::SimpleTimeManager;
use valkyrie_engine::engine::tree::components::{HasScore, HasVisits};
use valkyrie_engine::engine::tree::edges::AvgScoreEdge;
use valkyrie_engine::engine::tree::nodes::ClassicNode;
use valkyrie_engine::prelude::*;

fn main() {
    let engine = EngineBuilder::new()
        .best_move::<MaxQ>()
        .exploration::<Puct>()
        .expansion::<ClassicExpansion>()
        .simulation::<MaterialEvaluation>()
        .backpropagation::<BasicScalarBackpropagate>()
        .search::<BasicSequentialMCTS>()
        .time_manager::<SimpleTimeManager>()
        .node::<ClassicNode<AvgScoreEdge>>()
        .build();

    run(engine);
}

fn run<C: EngineConfig>(engine: Engine<C>)
where
    C::Search: SearchStrategy<C>,
    C::TimeManager: TimeManagerStrategy<C>,
    C::BestMove: BestMoveStrategy<C>,
    C::Node: HasVisits,
    C::Edge: HasScore + HasVisits,
{
    let registry = command_processors::registry::<C>();
    let args: Vec<String> = std::env::args().skip(1).collect();

    let parsed = cli::parse(&args, &|token| {
        registry
            .find(token)
            .filter(|command| command.cli)
            .map(|command| Lookup {
                name: command.name,
                primary: command.name == token,
            })
    });

    let mut shell = Shell {
        engine,
        input: Input::spawn(),
        registry,
        shutdown: false,
    };

    let cli = match parsed {
        Ok(cli) => cli,
        Err(violations) => {
            shell.engine.set_display(SimpleDisplay::default());
            print_violations(&shell.engine, &violations);
            std::process::exit(2);
        }
    };

    match cli.display {
        DisplayChoice::Simple => shell.engine.set_display(SimpleDisplay::default()),
        DisplayChoice::Minimal => shell.engine.set_display(MinimalDisplay::default()),
    }

    let mut ran_command = false;

    for item in &cli.items {
        match item {
            Item::SetOption { name, value } => {
                let mut tokens = vec!["setoption", "name", name.as_str()];

                if !value.is_empty() {
                    tokens.extend(["value", value.as_str()]);
                }

                shell.execute(&tokens);
            }
            Item::Command(tokens) => {
                let tokens: Vec<&str> = tokens.iter().map(String::as_str).collect();

                ran_command = true;
                shell.execute(&tokens);
            }
        }

        if shell.shutdown {
            return;
        }
    }

    if ran_command && !cli.interactive {
        return;
    }

    while !shell.shutdown {
        let Some(line) = shell.input.next() else {
            break;
        };

        let tokens: Vec<&str> = line.split_whitespace().collect();

        shell.execute(&tokens);
    }
}
