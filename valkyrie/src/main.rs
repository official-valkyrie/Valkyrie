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

mod command_processors;
mod input_wrapper;
mod reporters;

use std::sync::atomic::{AtomicBool, Ordering};

use command_processors::misc_processor::MiscProcessor;
use command_processors::uci_processor::UciProcessor;
use input_wrapper::InputWrapper;
use reporters::uci_reporter::UciReporter;
use valkyrie_engine::engine::builder::backpropagation_strategy::BasicScalarBackpropagate;
use valkyrie_engine::engine::builder::best_move_strategy::MaxQ;
use valkyrie_engine::engine::builder::expansion_strategy::ClassicExpansion;
use valkyrie_engine::engine::builder::exploration_strategy::Puct;
use valkyrie_engine::engine::builder::search_strategy::BasicSequentialMCTS;
use valkyrie_engine::engine::builder::simulation_strategy::MaterialEvaluation;
use valkyrie_engine::engine::builder::time_manager_strategy::SimpleTimeManager;
use valkyrie_engine::engine::tree::edges::AvgScoreEdge;
use valkyrie_engine::engine::tree::nodes::ClassicNode;
use valkyrie_engine::prelude::*;

fn main() {
    let mut engine = EngineBuilder::new()
        .best_move::<MaxQ>()
        .exploration::<Puct>()
        .expansion::<ClassicExpansion>()
        .simulation::<MaterialEvaluation>()
        .backpropagation::<BasicScalarBackpropagate>()
        .search::<BasicSequentialMCTS>()
        .time_manager::<SimpleTimeManager>()
        .node::<ClassicNode<AvgScoreEdge>>()
        .build();

    engine.set_reporter(UciReporter::default());

    let shutdown_token = AtomicBool::new(false);
    let mut input_wrapper = InputWrapper::new();

    while !shutdown_token.load(Ordering::SeqCst) {
        let cmd = match input_wrapper.get_input() {
            Some(cmd) => cmd,
            None => break,
        };

        let cmd = cmd.trim();

        let _ = MiscProcessor::execute(cmd, &engine)
            || UciProcessor::execute(cmd, &shutdown_token, &mut input_wrapper, &mut engine);
    }
}
