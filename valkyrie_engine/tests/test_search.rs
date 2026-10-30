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

use std::sync::{Arc, Mutex};

use valkyrie_chess::{ChessBoard, ChessPosition, FEN, Move};
use valkyrie_engine::{
    engine::builder::{
        backpropagation_strategy::BasicScalarBackpropagate, best_move_strategy::MaxQ,
        expansion_strategy::ClassicExpansion, exploration_strategy::Puct,
        search_strategy::BasicSequentialMCTS, simulation_strategy::MaterialEvaluation,
        time_manager_strategy::SimpleTimeManager,
    },
    engine::tree::{edges::AvgScoreEdge, nodes::ClassicNode},
    prelude::*,
};

macro_rules! engine {
    () => {
        EngineBuilder::new()
            .best_move::<MaxQ>()
            .exploration::<Puct>()
            .expansion::<ClassicExpansion>()
            .simulation::<MaterialEvaluation>()
            .backpropagation::<BasicScalarBackpropagate>()
            .search::<BasicSequentialMCTS>()
            .time_manager::<SimpleTimeManager>()
            .node::<ClassicNode<AvgScoreEdge>>()
            .build()
    };
}

fn position(fen: &str) -> ChessPosition {
    ChessPosition::from(ChessBoard::from(&FEN::from(fen)))
}

fn node_limit(nodes: u64) -> SearchLimits {
    let mut limits = SearchLimits::default();
    limits.set_nodes(Some(nodes));
    limits
}

fn is_legal(position: &ChessPosition, mv: Move) -> bool {
    let mut found = false;
    position
        .board()
        .map_legal_moves(|legal| found |= legal == mv);
    found
}

fn hard_limit_at<C: EngineConfig>(engine: &Engine<C>, limits: &SearchLimits, elapsed: u64) -> bool
where
    C::TimeManager: TimeManagerStrategy<C>,
{
    let params = engine.params().time_manager();
    C::TimeManager::new(limits, params, engine).hard_limit(elapsed, params, engine)
}

fn clock(time: u64, increment: Option<u64>, moves_to_go: Option<u64>) -> SearchLimits {
    let mut limits = SearchLimits::default();
    limits.set_time(Some(time));
    limits.set_increment(increment);
    limits.set_moves_to_go(moves_to_go);
    limits
}

#[test]
fn plays_a_legal_move_from_the_start_position() {
    let mut engine = engine!();
    let start = position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    engine.set_position(&start);

    let (mv, _) = engine.search(&node_limit(500));

    assert!(is_legal(&start, mv.unwrap()));
}

#[test]
fn node_limit_is_exact() {
    let mut engine = engine!();
    engine.set_position(&position(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    ));

    let (_, stats) = engine.search(&node_limit(100));

    assert_eq!(stats.iterations(), 100);
}

#[test]
fn finds_mate_in_one() {
    let mut engine = engine!();
    let board = position("6k1/5ppp/8/8/8/8/8/R3K3 w - - 0 1");
    engine.set_position(&board);

    let (mv, _) = engine.search(&node_limit(5000));

    assert_eq!(mv.unwrap().to_string(false), "a1a8");
}

#[test]
fn searches_can_be_repeated() {
    let mut engine = engine!();
    let board = position("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    engine.set_position(&board);

    for _ in 0..3 {
        let (mv, _) = engine.search(&node_limit(300));
        assert!(is_legal(&board, mv.unwrap()));
    }
}

#[test]
fn checkmated_root_has_no_move() {
    let mut engine = engine!();
    engine.set_position(&position("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1"));

    assert_eq!(engine.search(&node_limit(100)).0, None);
}

#[test]
fn stalemated_root_has_no_move() {
    let mut engine = engine!();
    engine.set_position(&position("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1"));

    assert_eq!(engine.search(&node_limit(100)).0, None);
}

#[test]
fn drawn_root_still_plays_a_legal_move() {
    let mut engine = engine!();
    let board = position("4k3/8/8/8/8/8/8/4K3 w - - 0 1");
    engine.set_position(&board);

    let (mv, stats) = engine.search(&node_limit(100));

    assert!(is_legal(&board, mv.unwrap()));
    assert_eq!(stats.iterations(), 0);
}

#[test]
fn hash_resize_keeps_search_working() {
    let mut engine = engine!();
    let board = position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    engine.set_position(&board);
    engine.set_option("Hash", "1").unwrap();

    let (mv, _) = engine.search(&node_limit(200_000));

    assert!(is_legal(&board, mv.unwrap()));
}

#[test]
fn move_time_is_the_hard_limit() {
    let engine = engine!();
    let mut limits = SearchLimits::default();
    limits.set_move_time(Some(500));

    assert!(!hard_limit_at(&engine, &limits, 499));
    assert!(hard_limit_at(&engine, &limits, 500));
}

#[test]
fn clock_is_split_over_twenty_moves_by_default() {
    let engine = engine!();
    let limits = clock(60_000, None, None);

    assert!(!hard_limit_at(&engine, &limits, 2989));
    assert!(hard_limit_at(&engine, &limits, 2990));
}

#[test]
fn zero_moves_to_go_counts_as_absent() {
    let engine = engine!();

    assert!(!hard_limit_at(&engine, &clock(60_000, None, Some(0)), 2989));
    assert!(hard_limit_at(&engine, &clock(60_000, None, Some(0)), 2990));
}

#[test]
fn moves_to_go_and_increment_change_the_budget() {
    let engine = engine!();

    assert!(hard_limit_at(&engine, &clock(1000, None, Some(1)), 990));
    assert!(!hard_limit_at(&engine, &clock(1000, None, Some(1)), 989));
    assert!(hard_limit_at(
        &engine,
        &clock(60_000, Some(2000), None),
        3990
    ));
    assert!(!hard_limit_at(
        &engine,
        &clock(60_000, Some(2000), None),
        3989
    ));
}

#[test]
fn budget_never_exceeds_the_clock_or_underflows() {
    let engine = engine!();

    assert!(!hard_limit_at(
        &engine,
        &clock(100, Some(1_000_000), Some(1)),
        89
    ));
    assert!(hard_limit_at(
        &engine,
        &clock(100, Some(1_000_000), Some(1)),
        90
    ));
    assert!(hard_limit_at(&engine, &clock(5, None, None), 0));
}

#[test]
fn without_a_clock_time_never_runs_out() {
    let engine = engine!();

    assert!(!hard_limit_at(&engine, &SearchLimits::default(), u64::MAX));
}

#[derive(Debug, Clone, Default)]
struct RecordingView {
    progress: Arc<Mutex<Vec<SearchReport>>>,
    finals: Arc<Mutex<Vec<SearchReport>>>,
}

impl View for RecordingView {
    fn wants_report(&self, stats: &SearchStats) -> bool {
        stats.iterations().is_multiple_of(100)
    }

    fn report(&self, report: &SearchReport) {
        self.progress.lock().unwrap().push(report.clone());
    }

    fn search_ended(&self, report: &SearchReport) {
        self.finals.lock().unwrap().push(report.clone());
    }
}

fn recorded_search(fen: &str, limits: &SearchLimits) -> RecordingView {
    let mut engine = engine!();
    let view = RecordingView::default();

    engine.set_position(&position(fen));
    engine.set_view(view.clone());
    engine.search(limits);

    view
}

#[test]
fn final_report_has_a_complete_root_move_table() {
    let view = recorded_search(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        &node_limit(2000),
    );
    let finals = view.finals.lock().unwrap();
    let report = finals.last().unwrap();

    assert_eq!(finals.len(), 1);
    assert_eq!(report.nodes, 2000);
    assert_eq!(report.lines.len(), 1);
    assert_eq!(report.root_moves.len(), 20);
    assert_eq!(
        report
            .root_moves
            .iter()
            .map(|root| root.visits)
            .sum::<u64>(),
        report.nodes
    );
    assert!(
        report
            .root_moves
            .windows(2)
            .all(|pair| pair[0].visits >= pair[1].visits)
    );
    assert_eq!(report.root_moves[0].mv, report.lines[0].moves[0]);
    assert!(
        report
            .root_moves
            .iter()
            .all(|root| root.score.is_some() == (root.visits > 0))
    );
    assert!(
        (report
            .root_moves
            .iter()
            .map(|root| root.policy)
            .sum::<f32>()
            - 1.0)
            .abs()
            < 1e-3
    );
}

#[test]
fn view_chooses_how_often_it_gets_progress_reports() {
    let view = recorded_search(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        &node_limit(1000),
    );

    let progress = view.progress.lock().unwrap();

    assert_eq!(progress.len(), 10);
    assert!(
        progress
            .iter()
            .all(|report| report.state == SearchState::Running)
    );
    assert!(
        progress
            .windows(2)
            .all(|pair| pair[0].nodes < pair[1].nodes)
    );
}

#[test]
fn report_carries_the_time_budget() {
    let mut move_time = SearchLimits::default();
    move_time.set_move_time(Some(50));

    let view = recorded_search("4k3/8/8/8/8/8/8/4K2R w K - 0 1", &move_time);
    assert_eq!(view.finals.lock().unwrap()[0].hard_limit_ms, Some(50));

    let view = recorded_search("4k3/8/8/8/8/8/8/4K2R w K - 0 1", &node_limit(100));
    assert_eq!(view.finals.lock().unwrap()[0].hard_limit_ms, None);
}

#[test]
fn report_scores_a_found_mate() {
    let view = recorded_search("6k1/5ppp/8/8/8/8/8/R3K3 w - - 0 1", &node_limit(5000));
    let finals = view.finals.lock().unwrap();
    let line = &finals[0].lines[0];

    assert_eq!(line.moves[0].to_string(false), "a1a8");
    assert_eq!(line.score.mate_in, Some(1));
}
