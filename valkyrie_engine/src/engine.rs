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

mod reporter;
mod search_limits;
mod search_state;
mod search_stats;
mod strategy_params;
mod policy_entry;

pub use builder::{EngineBuilder, EngineConfig, EngineParams, GeneralParams, SearchStrategy};
pub use reporter::{
    NoReporter,
    Reporter,
};
pub use search_limits::SearchLimits;
pub use search_stats::SearchStats;
pub use strategy_params::{EmptyParams, OptionError, StrategyParams, UciOptionType};

use std::{
    thread::{
        Scope,
        ScopedJoinHandle,
    },
    time::Duration,
};

use valkyrie_chess::{
    ChessPosition,
    Move,
};

use crate::engine::{
    builder::TimeManagerStrategy,
    search_state::{
        AtomicSearchState,
        SearchState,
    },
    tree::Tree,
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
    #[inline]
    pub fn params(&self) -> &EngineParams<C> {
        &self.params
    }

    #[inline]
    pub fn params_mut(&mut self) -> &mut EngineParams<C> {
        &mut self.params
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
    pub fn tree_mut(&mut self) -> &mut Tree<C::Node> {
        &mut self.tree
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
        self.search_state.transition(SearchState::Running, SearchState::Interrupted);
        self.search_state.transition(SearchState::Ended, SearchState::Interrupted);
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
{
    pub fn search(&self, limits: &SearchLimits) -> (Move, SearchStats) {
        self.search_state.set(SearchState::Running);
        self.run_search(limits)
    }

    pub fn spawn_search<'scope, 'env>(
        &'env self,
        scope: &'scope Scope<'scope, 'env>,
        limits: &'env SearchLimits,
    ) -> ScopedJoinHandle<'scope, (Move, SearchStats)> {
        self.search_state.set(SearchState::Running);
        scope.spawn(move || self.run_search(limits))
    }

    fn run_search(&self, limits: &SearchLimits) -> (Move, SearchStats) {
        self.reporter.search_started(limits, self);

        let (best_move, stats) = C::Search::execute(limits, self.params().search(), self);

        self.search_state.transition(SearchState::Running, SearchState::Ended);

        self.reporter.search_ended(&stats, self);

        if limits.is_infinite() {
            self.wait_for_interrupt();
        }

        self.reporter.best_move(best_move, self);

        self.search_state.set(SearchState::Idle);

        (best_move, stats)
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
    pub fn should_stop(
        &self,
        iterations: u64,
        limits: &SearchLimits,
        stats: &SearchStats,
        time_manager: &C::TimeManager,
    ) -> bool {
        if self.is_stopped() {
            return true;
        }

        if self.tree().is_full() {
            return true;
        }

        if limits.is_infinite() {
            return false;
        }

        if limits.nodes().is_some_and(|nodes| stats.iterations() >= nodes) {
            return true;
        }

        if limits.depth().is_some_and(|depth| stats.avg_depth() >= depth) {
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
