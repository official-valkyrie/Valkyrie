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

use valkyrie_chess::{ChessBoard, ChessPosition, FEN, Side};
use valkyrie_engine::prelude::*;

use crate::input_wrapper::InputWrapper;

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
        if args.is_empty() {
            println!("info string Incorrect command params");
            return;
        }

        let mut idx = 0;

        let fen = if args[idx] == "startpos" {
            idx += 1;
            FEN::start_position().to_string()
        } else if args[idx] == "fen" {
            idx += 1;
            let mut fen_parts = Vec::with_capacity(6);
            while idx < args.len() && args[idx] != "moves" {
                fen_parts.push(args[idx]);
                idx += 1;
            }
            fen_parts.join(" ")
        } else {
            println!("info string Invalid position command");
            return;
        };

        let violations = FEN::validate_fen(&fen);
        if !violations.is_empty() {
            for violation in violations {
                println!("info string Invalid FEN: {violation}");
            }
            return;
        }

        let mut chess_position = ChessPosition::from(ChessBoard::from(&FEN::from(fen)));

        if idx < args.len() && args[idx] == "moves" {
            idx += 1;
            for &mv_str in &args[idx..] {
                let mut legal_move = false;
                chess_position.board().clone().map_legal_moves(|legal_mv| {
                    if mv_str == legal_mv.to_string(engine.params().general().chess960()) {
                        legal_move = true;
                        chess_position.make_move_no_mask(legal_mv);
                    }
                });

                if !legal_move {
                    println!("info string Illegal move: {}", mv_str);
                    return;
                }
            }
        }

        engine.set_position(&chess_position);

        println!("info string Position set successfully");
    }

    fn set_option<C: EngineConfig>(args: &[&str], engine: &mut Engine<C>) {
        if args.len() < 2 || args[0] != "name" {
            println!("info string Incorrect command params");
            return;
        }

        let mut name = Vec::new();
        let mut value = Vec::new();
        let mut is_value = false;

        for &token in &args[1..] {
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

        let old_hash = engine.params().general().hash();

        if let Err(msg) = engine.params_mut().set_option(&name_str, &value_str) {
            println!("info string {msg}");
            return;
        }

        if name_str.eq_ignore_ascii_case("hash") {
            let hash_size = engine.params().general().hash();
            if hash_size != old_hash {
                engine.tree_mut().resize(hash_size as usize);
            }
        }

        if name_str.eq_ignore_ascii_case("clearhash") {
            engine.tree_mut().clear();
        }

        if is_value {
            println!("info string Option {name_str} has been set to {value_str}");
        } else {
            println!("info string Option {name_str} has been triggered");
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
        let limits = args_to_search_limits(args, engine.position().board().side());

        engine.tree_mut().clear();

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

fn args_to_search_limits(args: &[&str], stm: Side) -> SearchLimits {
    let mut limits = SearchLimits::default();

    for (idx, &arg) in args.iter().enumerate() {
        match arg {
            "wtime" => {
                if args.len() > idx + 1 && stm == Side::WHITE {
                    limits.set_time(args[idx + 1].parse::<u64>().ok());
                }
            }
            "winc" => {
                if args.len() > idx + 1 && stm == Side::WHITE {
                    limits.set_increment(args[idx + 1].parse::<u64>().ok());
                }
            }
            "btime" => {
                if args.len() > idx + 1 && stm == Side::BLACK {
                    limits.set_time(args[idx + 1].parse::<u64>().ok());
                }
            }
            "binc" => {
                if args.len() > idx + 1 && stm == Side::BLACK {
                    limits.set_increment(args[idx + 1].parse::<u64>().ok());
                }
            }
            "movestogo" => {
                if args.len() > idx + 1 {
                    limits.set_moves_to_go(args[idx + 1].parse::<u64>().ok());
                }
            }
            "movetime" => {
                if args.len() > idx + 1 {
                    limits.set_move_time(args[idx + 1].parse::<u64>().ok());
                }
            }
            "depth" => {
                if args.len() > idx + 1 {
                    limits.set_depth(args[idx + 1].parse::<u64>().ok());
                }
            }
            "nodes" => {
                if args.len() > idx + 1 {
                    limits.set_nodes(args[idx + 1].parse::<u64>().ok());
                }
            }
            "infinite" => limits.set_infinite(true),
            _ => continue,
        }
    }

    limits
}
