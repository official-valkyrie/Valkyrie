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

use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use valkyrie_chess::ChessBoard;
use valkyrie_engine::{
    engine::tree::components::{HasScore, HasVisits, Score},
    prelude::*,
};

const REPORT_INTERVAL: u64 = 1000;
const INTERVAL_CHECK_THRESHOLD: u64 = 128;
const PERFT_SEPARATOR: &str = "-----------------------------------------------------------";

#[derive(Debug, Default)]
pub struct SimpleDisplay {
    last_avg_depth: AtomicU64,
    last_max_depth: AtomicU64,
    last_report_time: AtomicU64,
}

impl SimpleDisplay {
    fn print_report<S: Score>(report: &SearchReport<S>) {
        for (pv_idx, pv) in report.lines.iter().enumerate() {
            let score = match pv.state().mate_in() {
                Some(moves) => format!("mate {moves}"),
                None => format!("cp {}", pv.score().cp()),
            };
            let moves: Vec<String> = pv
                .moves()
                .iter()
                .map(|mv| mv.to_string(report.chess960))
                .collect();

            println!(
                "info depth {} seldepth {} score {score} time {} nodes {} nps {} hashfull {} multipv {} pv {}",
                report.depth,
                report.seldepth,
                report.time_ms,
                report.nodes,
                report.nps,
                report.hashfull,
                pv_idx + 1,
                moves.join(" ")
            )
        }
    }

    fn should_report(&self, stats: &SearchStats) -> bool {
        stats.avg_depth() > self.last_avg_depth.load(Ordering::Relaxed)
            || stats.max_depth() > self.last_max_depth.load(Ordering::Relaxed)
            || (stats.iterations().is_multiple_of(INTERVAL_CHECK_THRESHOLD)
                && stats.elapsed_ms()
                    >= self.last_report_time.load(Ordering::Relaxed) + REPORT_INTERVAL)
    }

    fn report<C: EngineConfig>(&self, context: &SearchContext<C>, engine: &Engine<C>)
    where
        C::BestMove: BestMoveStrategy<C>,
        C::Node: HasVisits,
        C::Edge: HasScore + HasVisits,
    {
        let stats = context.stats();

        self.last_avg_depth
            .store(stats.avg_depth(), Ordering::Relaxed);
        self.last_max_depth
            .store(stats.max_depth(), Ordering::Relaxed);
        self.last_report_time
            .store(stats.elapsed_ms(), Ordering::Relaxed);

        Self::print_report(&engine.report(context));
    }
}

impl<C: EngineConfig> Display<C> for SimpleDisplay
where
    C::BestMove: BestMoveStrategy<C>,
    C::Node: HasVisits,
    C::Edge: HasScore + HasVisits,
{
    fn message(&self, _level: MessageLevel, text: &str) {
        println!("{text}")
    }

    fn identify(&self, name: &str, author: &str) {
        println!("id name {name}");
        println!("id author {author}");
    }

    fn option(&self, description: &str) {
        println!("{description}")
    }

    fn handshake_done(&self) {
        println!("uciok")
    }

    fn ready(&self) {
        println!("readyok")
    }

    fn board(&self, board: &ChessBoard) {
        println!("{board}")
    }

    fn perft_started(&self, board: &ChessBoard, depth: u8, bulk: bool) {
        println!();
        println!("{board}");
        println!("{PERFT_SEPARATOR}");
        println!("  Running PERFT");
        println!("  Depth: {depth}");
        println!("  Bulk: {bulk}");
        println!("  PEXT: {}", cfg!(target_feature = "bmi2"));
        println!("{PERFT_SEPARATOR}\n");
    }

    fn perft_move(&self, mv: Move, nodes: u64, engine: &Engine<C>) {
        println!(
            "  {} - {nodes}",
            mv.to_string(engine.params().general().chess960())
        )
    }

    fn perft_finished(&self, nodes: u64, elapsed: Duration) {
        let milliseconds = elapsed.as_millis().max(1);

        println!("\n{PERFT_SEPARATOR}");
        println!(
            "  Perft ended! {nodes} nodes, {milliseconds}ms, {} nps",
            u128::from(nodes) * 1000 / milliseconds
        );
        println!("{PERFT_SEPARATOR}\n");
    }

    fn bench_finished(&self, nodes: u64, elapsed: Duration) {
        let milliseconds = elapsed.as_millis().max(1);

        println!(
            "{nodes} nodes {} nps",
            u128::from(nodes) * 1000 / milliseconds
        );
    }

    fn search_started(&self, _context: &SearchContext<C>, _engine: &Engine<C>) {
        self.last_avg_depth.store(0, Ordering::Relaxed);
        self.last_max_depth.store(0, Ordering::Relaxed);
        self.last_report_time.store(0, Ordering::Relaxed);
    }

    fn search_update(&self, context: &SearchContext<C>, engine: &Engine<C>) {
        if self.should_report(context.stats()) {
            self.report(context, engine);
        }
    }

    fn search_ended(&self, context: &SearchContext<C>, engine: &Engine<C>) {
        self.report(context, engine);
    }

    fn best_move(&self, mv: Move, engine: &Engine<C>) {
        println!(
            "bestmove {}",
            mv.to_string(engine.params().general().chess960())
        )
    }
}
