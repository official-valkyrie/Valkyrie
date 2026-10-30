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

use valkyrie_chess::{ChessPosition, GameState};

use crate::{
    engine::{
        builder::SimulationStrategy,
        policy_entry::PolicyEntry,
        tree::components::{HasGameState, HasMove, HasVisits},
    },
    prelude::*,
};

#[derive(Debug)]
pub struct BasicSequentialMCTS;

impl Strategy for BasicSequentialMCTS {
    type Params = EmptyParams;
}

impl<C: EngineConfig> SearchStrategy<C> for BasicSequentialMCTS
where
    C::TimeManager: TimeManagerStrategy<C>,
    C::Exploration: ExplorationStrategy<C>,
    C::Expansion: ExpansionStrategy<C>,
    C::Simulation: SimulationStrategy<C>,
    C::Backpropagation: BackpropagationStrategy<C>,
    C::Node: HasVisits,
{
    fn execute(context: &SearchContext<C>, _params: &Self::Params, engine: &Engine<C>) {
        Searcher::new(context, engine).run();
    }
}

struct Searcher<'a, C: EngineConfig> {
    context: &'a SearchContext<'a, C>,
    engine: &'a Engine<C>,
    depth: u64,
}

impl<'a, C: EngineConfig> Searcher<'a, C>
where
    C::TimeManager: TimeManagerStrategy<C>,
    C::Exploration: ExplorationStrategy<C>,
    C::Expansion: ExpansionStrategy<C>,
    C::Simulation: SimulationStrategy<C>,
    C::Backpropagation: BackpropagationStrategy<C>,
    C::Node: HasVisits,
{
    fn new(context: &'a SearchContext<'a, C>, engine: &'a Engine<C>) -> Self {
        Self {
            context,
            engine,
            depth: 0,
        }
    }

    fn run(&mut self) {
        let engine = self.engine;
        let root_node = engine.tree().root_node();

        if root_node.edge_count() == 0 {
            root_node.set_game_state(engine.position().game_state(engine.position()));

            if !root_node.is_terminal() {
                self.expand_node(root_node, engine.position());
            }
        }

        if root_node.edge_count() > 0 {
            self.main_thread_search();
        }
    }

    fn main_thread_search(&mut self) {
        let engine = self.engine;
        let stats = self.context.stats();

        loop {
            let mut position = *engine.position();
            self.depth = 0;

            if self
                .search_step::<true>(engine.tree().root_index(), &mut position)
                .is_none()
            {
                break;
            }

            stats.add_iteration(self.depth);

            engine.display().search_update(self.context, engine);

            if engine.should_stop(self.context) {
                break;
            }
        }
    }

    fn search_step<const ROOT: bool>(
        &mut self,
        current_node_idx: NodeIndex,
        position: &mut ChessPosition,
    ) -> Option<<C::Simulation as SimulationStrategy<C>>::Output> {
        let engine = self.engine;
        let current_node = &engine.tree()[current_node_idx];

        let (payload, edge_idx) = if !ROOT
            && (current_node.visits() == 0
                || current_node.edge_count() == 0
                || current_node.is_terminal())
        {
            if current_node.visits() == 0 {
                current_node.set_game_state(position.game_state(engine.position()));
            }

            let payload = self.simulate_node(&[], current_node, position);

            if current_node.visits() == 0 && !current_node.is_terminal() {
                self.expand_node(current_node, position);
            }

            (payload, None)
        } else {
            let edge_idx =
                C::Exploration::execute(current_node, engine.params().exploration(), engine);

            let child_idx = engine
                .tree()
                .get_or_create_child(current_node_idx, edge_idx)?;
            let mv = current_node.edges()[edge_idx].mv();

            position.make_move(mv);

            self.depth = self.depth.saturating_add(1);

            let payload = self.search_step::<false>(child_idx, position)?;

            (payload, Some(edge_idx))
        };

        let backed_up = C::Backpropagation::execute(
            payload,
            current_node,
            edge_idx,
            engine.params().backpropagation(),
            engine,
        );

        Some(backed_up.flipped())
    }

    fn expand_node(&self, node: &C::Node, position: &ChessPosition) {
        let mut policy_distribution = Vec::with_capacity(35);
        position.board().map_legal_moves(|mv| {
            policy_distribution.push(PolicyEntry::new(mv, 1.0));
        });

        C::Expansion::execute(
            &mut policy_distribution,
            node,
            self.engine.params().expansion(),
            self.engine,
        );
    }

    fn simulate_node(
        &self,
        raw_evals: &[f32],
        node: &C::Node,
        position: &ChessPosition,
    ) -> <C::Simulation as SimulationStrategy<C>>::Output {
        match node.game_state() {
            GameState::Won(_) => Payload::WIN,
            GameState::Lost(_) => Payload::LOSS,
            GameState::Drew => Payload::DRAW,
            GameState::Ongoing => C::Simulation::execute(
                raw_evals,
                position,
                self.engine.params().simulation(),
                self.engine,
            ),
        }
    }
}
