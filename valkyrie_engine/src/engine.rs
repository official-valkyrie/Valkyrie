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

mod macros;

pub mod builder;
pub mod tree;

mod payload;
mod policy_entry;
mod pv;
mod reporter;
mod search_context;
mod search_limits;
mod search_state;
mod search_stats;
mod strategy_params;

pub use builder::{EngineBuilder, EngineConfig, EngineParams, GeneralParams, SearchStrategy};
pub use payload::{HasWdl, Payload, WinChance};
pub use pv::Pv;
pub use reporter::{NoReporter, Reporter};
pub use search_context::SearchContext;
pub use search_limits::SearchLimits;
pub use search_stats::SearchStats;
pub use strategy_params::{
    EmptyParams, OptionNames, OptionViolation, StrategyParams, UciOptionType,
};

use std::{
    thread::{Scope, ScopedJoinHandle},
    time::Duration,
};

use valkyrie_chess::{ChessBoard, ChessPosition, FEN, GameState, Move};

use crate::engine::{
    builder::{BestMoveStrategy, TimeManagerStrategy},
    search_state::{AtomicSearchState, SearchState},
    tree::{
        Tree,
        components::{HasChild, HasGameState, HasMove, HasScore, HasVisits, NodeType},
    },
};

const HARD_LIMIT_CHECK_THRESHOLD: u64 = 128;
const SOFT_LIMIT_CHECK_THRESHOLD: u64 = 4096;
const STOP_POLL_INTERVAL: Duration = Duration::from_millis(1);

#[derive(Debug)]
pub struct Engine<C: EngineConfig> {
    params: EngineParams<C>,
    position: ChessPosition,
    search_state: AtomicSearchState,
    tree: Tree<C::Node>,
    reporter: Box<dyn Reporter<C>>,
}

impl<C: EngineConfig> Engine<C> {
    pub(crate) fn new(params: EngineParams<C>) -> Self {
        let tree = Tree::new(params.general().hash() as usize);

        Self {
            params,
            position: ChessPosition::from(ChessBoard::from(&FEN::start_position())),
            search_state: AtomicSearchState::default(),
            tree,
            reporter: Box::new(NoReporter),
        }
    }

    #[inline]
    pub fn params(&self) -> &EngineParams<C> {
        &self.params
    }

    pub fn set_option(&mut self, name: &str, value: &str) -> Result<(), OptionViolation> {
        if name.is_empty() {
            return Err(OptionViolation::MissingName);
        }

        let hash = self.params.general().hash();

        self.params.set_option(name, value)?;

        if self.params.general().hash() != hash {
            self.tree.resize(self.params.general().hash() as usize);
        }

        if name.eq_ignore_ascii_case("ClearHash") {
            self.tree.clear();
        }

        Ok(())
    }

    #[inline]
    pub fn position(&self) -> &ChessPosition {
        &self.position
    }

    #[inline]
    pub fn set_position(&mut self, position: &ChessPosition) {
        self.position = *position;
    }

    #[inline]
    pub fn tree(&self) -> &Tree<C::Node> {
        &self.tree
    }

    #[inline]
    pub fn reported_nodes(&self, stats: &SearchStats) -> u64 {
        if self.params.general().iters_as_nodes() {
            stats.iterations()
        } else {
            stats.cumulative_depth()
        }
    }

    #[inline]
    pub fn stop(&self) {
        self.search_state
            .transition(SearchState::Running, SearchState::Interrupted);
        self.search_state
            .transition(SearchState::Ended, SearchState::Interrupted);
    }

    #[inline]
    pub fn is_running(&self) -> bool {
        self.search_state.is(SearchState::Running)
    }

    #[inline]
    pub fn is_ended(&self) -> bool {
        self.search_state.is(SearchState::Ended)
    }

    #[inline]
    pub fn is_interrupted(&self) -> bool {
        self.search_state.is(SearchState::Interrupted)
    }

    #[inline]
    pub fn is_stopped(&self) -> bool {
        !self.is_running()
    }

    #[inline]
    pub fn reporter(&self) -> &dyn Reporter<C> {
        self.reporter.as_ref()
    }

    pub fn set_reporter(&mut self, reporter: impl Reporter<C> + 'static) {
        self.reporter = Box::new(reporter);
    }

    #[inline]
    pub fn print(&self, msg: &str) {
        self.reporter.print(msg)
    }
}

impl<C: EngineConfig> Engine<C>
where
    C::Search: SearchStrategy<C>,
    C::TimeManager: TimeManagerStrategy<C>,
    C::BestMove: BestMoveStrategy<C>,
{
    pub fn search(&self, limits: &SearchLimits) -> (Option<Move>, SearchStats) {
        self.search_state.set(SearchState::Running);
        self.run_search(SearchContext::new(limits, self))
    }

    pub fn spawn_search<'scope, 'env>(
        &'env self,
        scope: &'scope Scope<'scope, 'env>,
        limits: &'env SearchLimits,
    ) -> ScopedJoinHandle<'scope, (Option<Move>, SearchStats)> {
        self.search_state.set(SearchState::Running);
        let context = SearchContext::new(limits, self);
        scope.spawn(move || self.run_search(context))
    }

    fn run_search(&self, context: SearchContext<C>) -> (Option<Move>, SearchStats) {
        let best_move = self.choose_move(&context);

        self.search_state
            .transition(SearchState::Running, SearchState::Ended);

        if let Some(best_move) = best_move {
            if context.limits().is_infinite() {
                self.wait_for_interrupt();
            }

            self.reporter.best_move(best_move, self);
        }

        self.search_state.set(SearchState::Idle);

        (best_move, context.into_stats())
    }

    fn choose_move(&self, context: &SearchContext<C>) -> Option<Move> {
        let Some(first_move) = self.first_legal_move() else {
            self.print("Illegal position: no legal moves");
            return None;
        };

        if self.position.game_state(&self.position) != GameState::Ongoing {
            return Some(first_move);
        }

        self.tree.clear();

        self.reporter.search_started(context.limits(), self);

        C::Search::execute(context, self.params().search(), self);

        self.reporter.search_ended(context.stats(), self);

        Some(self.best_move().unwrap_or(first_move))
    }

    fn first_legal_move(&self) -> Option<Move> {
        let mut first_move = None;

        self.position.board().map_legal_moves(|mv| {
            if first_move.is_none() {
                first_move = Some(mv);
            }
        });

        first_move
    }

    fn wait_for_interrupt(&self) {
        while !self.is_interrupted() {
            std::thread::sleep(STOP_POLL_INTERVAL);
        }
    }
}

impl<C: EngineConfig> Engine<C>
where
    C::TimeManager: TimeManagerStrategy<C>,
{
    pub fn should_stop(&self, context: &SearchContext<C>) -> bool {
        let (limits, stats, time_manager) =
            (context.limits(), context.stats(), context.time_manager());
        let iterations = stats.iterations();

        if self.is_stopped() {
            return true;
        }

        if self.tree().is_full() {
            return true;
        }

        if limits.is_infinite() {
            return false;
        }

        if limits.nodes().is_some_and(|nodes| iterations >= nodes) {
            return true;
        }

        if limits
            .depth()
            .is_some_and(|depth| stats.avg_depth() >= depth)
        {
            return true;
        }

        if iterations.is_multiple_of(HARD_LIMIT_CHECK_THRESHOLD)
            && time_manager.hard_limit(stats.elapsed_ms(), self.params().time_manager(), self)
        {
            return true;
        }

        iterations.is_multiple_of(SOFT_LIMIT_CHECK_THRESHOLD)
            && time_manager.soft_limit(stats.elapsed_ms(), self.params().time_manager(), self)
    }
}

impl<C: EngineConfig> Engine<C>
where
    C::BestMove: BestMoveStrategy<C>,
{
    pub fn best_move(&self) -> Option<Move> {
        let root = self.tree().root_node();

        C::BestMove::execute(root, 0, self.params().best_move(), self)
            .map(|edge_idx| root.edges()[edge_idx].mv())
    }

    pub fn pv(&self, pv_idx: usize) -> Option<Pv<<C::Edge as HasScore>::Score>>
    where
        C::Node: HasVisits,
        C::Edge: HasScore + HasVisits,
    {
        let tree = self.tree();
        let root = tree.root_node();
        let root_edge = C::BestMove::execute(root, pv_idx, self.params().best_move(), self)?;

        let (score, state, visit_percent) = {
            let edges = root.edges();
            let edge = &edges[root_edge];
            let state = if edge.has_child() {
                tree[edge.child()].game_state().for_parent()
            } else {
                GameState::Ongoing
            };

            (
                edge.score(),
                state,
                edge.visits() as f32 * 100.0 / root.visits().max(1) as f32,
            )
        };

        let mut moves = Vec::new();
        let mut node_idx = tree.root_index();
        let mut next_edge = Some(root_edge);

        while let Some(edge_idx) = next_edge {
            let edges = tree[node_idx].edges();
            let edge = &edges[edge_idx];

            moves.push(edge.mv());

            if !edge.has_child() {
                break;
            }

            node_idx = edge.child();
            next_edge = C::BestMove::execute(&tree[node_idx], 0, self.params().best_move(), self);
        }

        Some(Pv::new(moves, score, state, visit_percent))
    }
}
