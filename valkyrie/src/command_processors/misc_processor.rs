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

use std::process::Command as Process;

use valkyrie_chess::DEFAULT_PERFT_DEPTH;
use valkyrie_engine::prelude::*;

use crate::{
    command_processors::{perft_args::parse_perft, print_violations},
    commands::{Category, Command, Shell},
};

pub struct MiscProcessor;
impl MiscProcessor {
    pub fn commands<C: EngineConfig>() -> Vec<Command<C>> {
        vec![
            Command {
                name: "draw",
                aliases: &["d"],
                category: Category::Tools,
                cli: true,
                during_search: false,
                usage: "draw",
                description: "Show the current board",
                run: Self::draw,
            },
            Command {
                name: "tunables",
                aliases: &[],
                category: Category::Tools,
                cli: true,
                during_search: false,
                usage: "tunables",
                description: "Print the tunable parameters in tuner format",
                run: Self::tunables,
            },
            Command {
                name: "perft",
                aliases: &[],
                category: Category::Tools,
                cli: true,
                during_search: false,
                usage: "perft [DEPTH]",
                description: "Count the legal-move tree with bulk counting",
                run: Self::perft::<C, true>,
            },
            Command {
                name: "perft_no_bulk",
                aliases: &[],
                category: Category::Tools,
                cli: true,
                during_search: false,
                usage: "perft_no_bulk [DEPTH]",
                description: "Count the legal-move tree without bulk counting",
                run: Self::perft::<C, false>,
            },
            Command {
                name: "bench",
                aliases: &[],
                category: Category::Tools,
                cli: true,
                during_search: false,
                usage: "bench",
                description: "Run the benchmark (not implemented yet)",
                run: Self::bench,
            },
            Command {
                name: "clear",
                aliases: &["cls"],
                category: Category::Tools,
                cli: false,
                during_search: false,
                usage: "clear",
                description: "Clear the terminal",
                run: Self::clear_terminal_screen,
            },
        ]
    }

    fn draw<C: EngineConfig>(shell: &mut Shell<C>, _args: &[&str]) {
        let engine = shell.engine();

        engine.view().board(engine.position().board());
    }

    fn tunables<C: EngineConfig>(shell: &mut Shell<C>, _args: &[&str]) {
        let engine = shell.engine();

        engine.params().print_tunables(&|line| engine.print(line));
    }

    fn perft<C: EngineConfig, const BULK: bool>(shell: &mut Shell<C>, args: &[&str]) {
        let engine = shell.engine();

        let depth = match parse_perft(args) {
            Ok(depth) => depth,
            Err(violations) => {
                print_violations(engine, &violations);
                return;
            }
        };

        let depth = depth.unwrap_or(DEFAULT_PERFT_DEPTH);
        let chess960 = engine.params().general().chess960();
        let board = engine.position().board();

        engine.view().perft_started(board, depth, BULK);

        let (nodes, duration) = valkyrie_chess::perft::<BULK>(board, Some(depth), |mv, nodes| {
            engine.view().perft_move(mv, nodes, chess960)
        });

        engine.view().perft_finished(nodes, duration);
    }

    fn bench<C: EngineConfig>(_shell: &mut Shell<C>, _args: &[&str]) {}

    fn clear_terminal_screen<C: EngineConfig>(_shell: &mut Shell<C>, _args: &[&str]) {
        if cfg!(target_os = "windows") {
            Process::new("cmd")
                .args(["/c", "cls"])
                .spawn()
                .expect("cls command failed to start")
                .wait()
                .expect("failed to wait");
        } else {
            Process::new("clear")
                .spawn()
                .expect("clear command failed to start")
                .wait()
                .expect("failed to wait");
        };
    }
}
