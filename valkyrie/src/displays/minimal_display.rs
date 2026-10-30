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
    engine::tree::components::{HasScore, HasVisits},
    prelude::*,
};

use crate::displays::simple_display::SimpleDisplay;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct MinimalDisplay(SimpleDisplay);

impl<C: EngineConfig> Display<C> for MinimalDisplay
where
    C::BestMove: BestMoveStrategy<C>,
    C::Node: HasVisits,
    C::Edge: HasScore + HasVisits,
{
    fn identify(&self, name: &str, author: &str) {
        Display::<C>::identify(&self.0, name, author)
    }

    fn option(&self, description: &str) {
        Display::<C>::option(&self.0, description)
    }

    fn handshake_done(&self) {
        Display::<C>::handshake_done(&self.0)
    }

    fn ready(&self) {
        Display::<C>::ready(&self.0)
    }

    fn search_ended(&self, context: &SearchContext<C>, engine: &Engine<C>) {
        self.0.search_ended(context, engine)
    }

    fn best_move(&self, mv: Move, engine: &Engine<C>) {
        self.0.best_move(mv, engine)
    }
}
