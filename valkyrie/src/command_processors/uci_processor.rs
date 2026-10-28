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

use core::str;
use std::sync::atomic::{AtomicBool, Ordering};

use valkyrie_engine::prelude::*;

use crate::{
    command_processors::{go_args::parse_go, position_args::parse_position, print_violations},
    input_wrapper::InputWrapper,
};

pub struct UciProcessor;
impl UciProcessor {
    pub fn execute<C: EngineConfig>(
        cmd: &str,
        shutdown_token: &AtomicBool,
        input_wrapper: &mut InputWrapper,
        engine: &mut Engine<C>,
    ) -> bool
    where
        C::Search: SearchStrategy<C>,
    {
        let tokens: Vec<&str> = cmd.split_whitespace().collect();

        if tokens.len() == 0 {
            return false;
        }

        match tokens[0] {
            "uci" => Self::uci(engine),
            "isready" => println!("readyok"),
            "position" => Self::position(&tokens[1..], engine),
            "setoption" => Self::set_option(&tokens[1..], engine),
            "quit" | "q" => shutdown_token.store(true, Ordering::SeqCst),
            "go" => Self::go(&tokens[1..], engine, input_wrapper, shutdown_token),
            _ => return false,
        }

        true
    }

    fn uci<C: EngineConfig>(engine: &mut Engine<C>) {
        println!("id name {}", env!("ENGINE_NAME"));
        println!("id author {}", env!("CARGO_PKG_AUTHORS"));
        engine.params().print_options();
        println!("uciok");
    }

    fn position<C: EngineConfig>(args: &[&str], engine: &mut Engine<C>) {
        match parse_position(args, engine.params().general().chess960()) {
            Ok(position) => {
                engine.set_position(&position);
                engine.print("Position set successfully");
            }
            Err(violations) => print_violations(engine, &violations),
        }
    }

    fn set_option<C: EngineConfig>(args: &[&str], engine: &mut Engine<C>) {
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

    fn go<C: EngineConfig>(
        args: &[&str],
        engine: &mut Engine<C>,
        input_wrapper: &mut InputWrapper,
        shutdown_token: &AtomicBool,
    ) where
        C::Search: SearchStrategy<C>,
    {
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
                let cmd = match input_wrapper.get_input_no_queue() {
                    Some(cmd) => cmd,
                    None => {
                        engine.stop();
                        shutdown_token.store(true, Ordering::Relaxed);
                        break;
                    }
                };

                match cmd.trim() {
                    "isready" => println!("readyok"),
                    "stop" | "s" => {
                        engine.stop();
                        break;
                    }
                    "quit" | "q" => {
                        engine.stop();
                        shutdown_token.store(true, Ordering::Relaxed);
                        break;
                    }
                    _ => input_wrapper.push_back(cmd),
                }
            }
        });
    }
}
