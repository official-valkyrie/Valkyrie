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
mod search_context;
mod search_limits;
mod search_report;
mod search_state;
mod search_stats;
mod strategy_params;
mod view;

pub use builder::{EngineBuilder, EngineConfig, EngineParams, GeneralParams, SearchStrategy};
pub use payload::{HasWdl, Payload, WinChance};
pub use pv::Pv;
pub use search_context::SearchContext;
pub use search_limits::SearchLimits;
pub use search_report::{ReportLine, ReportScore, RootMove, SearchReport};
pub use search_state::SearchState;
pub use search_stats::SearchStats;
pub use strategy_params::{
    EmptyParams, OptionKind, OptionNames, OptionSpec, OptionViolation, StrategyParams,
    UciOptionType,
};
pub use view::{MessageLevel, NoView, View};

use std::{
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use valkyrie_chess::{ChessBoard, ChessPosition, FEN, GameState, Move};

use crate::engine::{
    builder::{BestMoveStrategy, TimeManagerStrategy},
    search_state::AtomicSearchState,
    tree::{
        NodeKey, Tree,
        components::{HasChild, HasGameState, HasMove, HasPolicy, HasScore, HasVisits},
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
    tree: C::Tree,
    view: Box<dyn View>,
}

impl<C: EngineConfig> Engine<C> {
    pub(crate) fn new(params: EngineParams<C>) -> Self {
        let tree = C::Tree::new(params.general().hash() as usize);

        Self {
            params,
            position: ChessPosition::from(ChessBoard::from(&FEN::start_position())),
            search_state: AtomicSearchState::default(),
            tree,
            view: Box::new(NoView),
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
    pub fn tree(&self) -> &C::Tree {
        &self.tree
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
    pub fn search_state(&self) -> SearchState {
        self.search_state.get()
    }

    pub fn search_update(&self, context: &SearchContext<C>) {
        if self.view.wants_report(context.stats()) {
            self.view.report(&context.report(self));
        }
    }

    #[inline]
    pub fn view(&self) -> &dyn View {
        self.view.as_ref()
    }

    pub fn set_view(&mut self, view: impl View + 'static) {
        self.view = Box::new(view);
    }

    #[inline]
    pub fn print(&self, text: &str) {
        self.view.message(MessageLevel::Info, text)
    }

    #[inline]
    pub fn print_error(&self, text: &str) {
        self.view.message(MessageLevel::Error, text)
    }
}

impl<C: EngineConfig> Engine<C>
where
    C::Search: SearchStrategy<C>,
    C::TimeManager: TimeManagerStrategy<C>,
    C::BestMove: BestMoveStrategy<C>,
    C::Node: HasVisits,
    C::Edge: HasScore + HasVisits + HasPolicy,
{
    pub fn search(&mut self, limits: &SearchLimits) -> (Option<Move>, SearchStats) {
        self.tree.reroot(NodeKey::from(&self.position));
        self.search_state.set(SearchState::Running);
        self.run_search(SearchContext::new(limits, self, Self::report))
    }

    pub fn spawn_search(
        self: &mut Arc<Self>,
        limits: SearchLimits,
    ) -> JoinHandle<(Option<Move>, SearchStats)> {
        let engine = Arc::get_mut(self).expect("a search is already running");
        engine.tree.reroot(NodeKey::from(&engine.position));
        self.search_state.set(SearchState::Running);
        let engine = Arc::clone(self);

        thread::spawn(move || {
            let context = SearchContext::new(&limits, &engine, Self::report);
            engine.run_search(context)
        })
    }

    fn run_search(&self, context: SearchContext<C>) -> (Option<Move>, SearchStats) {
        let best_move = self.choose_move(&context);

        self.search_state
            .transition(SearchState::Running, SearchState::Ended);

        if let Some(best_move) = best_move {
            if context.limits().is_infinite() {
                self.wait_for_interrupt();
            }

            self.view
                .best_move(best_move, self.params.general().chess960());
        }

        self.search_state.set(SearchState::Idle);

        (best_move, context.into_stats())
    }

    fn choose_move(&self, context: &SearchContext<C>) -> Option<Move> {
        let Some(first_move) = self.first_legal_move() else {
            self.print_error("Illegal position: no legal moves");
            return None;
        };

        if self.position.game_state(&self.position) != GameState::Ongoing {
            return Some(first_move);
        }

        self.view.search_started(context.limits());

        C::Search::execute(context, self.params().search(), self);

        self.view.search_ended(&context.report(self));

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
            .map(|edge_idx| self.tree[edge_idx].mv())
    }

    pub fn report(&self, context: &SearchContext<C>) -> SearchReport
    where
        C::TimeManager: TimeManagerStrategy<C>,
        C::Node: HasVisits,
        C::Edge: HasScore + HasVisits + HasPolicy,
    {
        let stats = context.stats();
        let time_ms = stats.elapsed_ms();
        let nodes = stats.iterations();
        let time_manager = context.time_manager();

        SearchReport {
            limits: context.limits().clone(),
            root: *self.position.board(),
            chess960: self.params.general().chess960(),
            state: self.search_state(),
            time_ms,
            soft_limit_ms: time_manager.soft_limit_ms(),
            hard_limit_ms: time_manager.hard_limit_ms(),
            depth: stats.avg_depth(),
            seldepth: stats.max_depth(),
            nodes,
            nps: (u128::from(nodes) * 1000 / u128::from(time_ms.max(1))) as u64,
            hashfull: self.tree.used_bytes() * 1000 / self.tree.total_bytes().max(1),
            lines: (0..)
                .map_while(|pv_idx| self.pv(pv_idx))
                .take(self.params.general().multi_pv())
                .map(|pv| ReportLine {
                    moves: pv.moves().to_vec(),
                    score: ReportScore::new(pv.score(), pv.state()),
                    visit_percent: pv.visit_percent(),
                })
                .collect(),
            root_moves: self.root_moves(),
        }
    }

    fn root_moves(&self) -> Vec<RootMove>
    where
        C::Node: HasVisits,
        C::Edge: HasScore + HasVisits + HasPolicy,
    {
        let tree = self.tree();
        let root = tree.root_node();
        let root_visits = root.visits().max(1) as f32;

        let mut moves: Vec<RootMove> = tree
            .edges(root)
            .iter()
            .map(|edge| {
                let visits = edge.visits();
                let state = if edge.has_child() {
                    tree[edge.child()].game_state().for_parent()
                } else {
                    GameState::Ongoing
                };

                RootMove {
                    mv: edge.mv(),
                    visits,
                    visit_percent: visits as f32 * 100.0 / root_visits,
                    policy: edge.policy(),
                    score: (visits > 0).then(|| ReportScore::new(edge.score(), state)),
                }
            })
            .collect();

        moves.sort_by(|lhs, rhs| rhs.visits.cmp(&lhs.visits));
        moves
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
            let edge = &tree[root_edge];
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
        let mut next_edge = Some(root_edge);

        while let Some(edge_idx) = next_edge {
            let edge = &tree[edge_idx];

            moves.push(edge.mv());

            if !edge.has_child() {
                break;
            }

            next_edge =
                C::BestMove::execute(&tree[edge.child()], 0, self.params().best_move(), self);
        }

        Some(Pv::new(moves, score, state, visit_percent))
    }
}
