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

use std::process::Command;

use valkyrie_chess::DEFAULT_PERFT_DEPTH;
use valkyrie_engine::prelude::*;

use crate::command_processors::{perft_args::parse_perft, print_violations};

pub struct MiscProcessor;
impl MiscProcessor {
    pub fn execute<C: EngineConfig>(cmd: &str, engine: &Engine<C>) -> bool {
        let tokens: Vec<&str> = cmd.split_whitespace().collect();

        if tokens.len() == 0 {
            return false;
        }

        match tokens[0] {
            "draw" | "d" => engine.print(&engine.position().board().to_string()),
            "tunables" => engine.params().print_tunables(&|line| engine.print(line)),
            "perft" => Self::perft::<_, true>(&tokens[1..], engine),
            "perft_no_bulk" => Self::perft::<_, false>(&tokens[1..], engine),
            "bench" => Self::bench(&tokens[1..], engine),
            "clear" | "cls" => Self::clear_terminal_screen(),
            _ => return false,
        }

        true
    }

    fn perft<C: EngineConfig, const BULK: bool>(args: &[&str], engine: &Engine<C>) {
        let depth = match parse_perft(args) {
            Ok(depth) => depth,
            Err(violations) => {
                print_violations(engine, &violations);
                return;
            }
        };

        engine.print("");
        engine.print(&engine.position().board().to_string());

        engine.print("-----------------------------------------------------------");
        engine.print("  Running PERFT");
        engine.print(&format!(
            "  Depth: {}",
            depth.unwrap_or(DEFAULT_PERFT_DEPTH)
        ));
        engine.print(&format!("  Bulk: {BULK}"));
        engine.print(&format!("  PEXT: {}", cfg!(target_feature = "bmi2")));
        engine.print("-----------------------------------------------------------\n");

        let chess960 = engine.params().general().chess960();
        let (result, duration) =
            valkyrie_chess::perft::<BULK>(engine.position().board(), depth, |mv, nodes| {
                engine.print(&format!("  {} - {nodes}", mv.to_string(chess960)))
            });

        let miliseconds = duration.as_millis().max(1);

        engine.print("\n-----------------------------------------------------------");
        engine.print(&format!(
            "  Perft ended! {result} nodes, {miliseconds}ms, {} nps",
            result as u128 * 1000 / miliseconds
        ));
        engine.print("-----------------------------------------------------------\n");
    }

    fn bench<C: EngineConfig>(_args: &[&str], _engine: &Engine<C>) {}

    fn clear_terminal_screen() {
        if cfg!(target_os = "windows") {
            Command::new("cmd")
                .args(["/c", "cls"])
                .spawn()
                .expect("cls command failed to start")
                .wait()
                .expect("failed to wait");
        } else {
            Command::new("clear")
                .spawn()
                .expect("clear command failed to start")
                .wait()
                .expect("failed to wait");
        };
    }
}
