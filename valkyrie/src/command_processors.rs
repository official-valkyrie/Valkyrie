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

pub mod depth_args;
pub mod general_processor;
pub mod go_args;
pub mod misc_processor;
pub mod position_args;
pub mod uci_processor;

use std::fmt::Display;

use valkyrie_engine::{
    engine::tree::components::{HasPolicy, HasScore, HasVisits},
    prelude::*,
};

use crate::commands::Registry;
use general_processor::GeneralProcessor;
use misc_processor::MiscProcessor;
use uci_processor::UciProcessor;

pub fn registry<C: EngineConfig>() -> Registry<C>
where
    C::Search: SearchStrategy<C>,
    C::TimeManager: TimeManagerStrategy<C>,
    C::BestMove: BestMoveStrategy<C>,
    C::Node: HasVisits,
    C::Edge: HasScore + HasVisits + HasPolicy,
{
    let mut commands = UciProcessor::commands();
    commands.extend(MiscProcessor::commands());
    commands.extend(GeneralProcessor::commands());

    Registry::new(commands)
}

pub fn print_violations<C: EngineConfig, V: Display>(engine: &Engine<C>, violations: &[V]) {
    for violation in violations {
        engine.print_error(&violation.to_string());
    }
}
