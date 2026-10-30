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

use std::sync::atomic::{AtomicU64, Ordering};

use valkyrie_engine::{
    engine::tree::components::{HasScore, HasVisits},
    prelude::*,
};

const REPORT_INTERVAL: u64 = 1000;
const INTERVAL_CHECK_THRESHOLD: u64 = 128;

#[derive(Debug, Default)]
pub struct UciReporter {
    last_avg_depth: AtomicU64,
    last_max_depth: AtomicU64,
    last_report_time: AtomicU64,
}

impl UciReporter {
    pub fn print_report<C: EngineConfig>(stats: &SearchStats, engine: &Engine<C>)
    where
        C::BestMove: BestMoveStrategy<C>,
        C::Node: HasVisits,
        C::Edge: HasScore + HasVisits,
    {
        let time = stats.elapsed_ms();
        let depth = stats.avg_depth();
        let max_depth = stats.max_depth();
        let nodes = engine.reported_nodes(stats);
        let nps = nodes as u128 * 1000 / time.max(1) as u128;
        let hashfull = engine.tree().len() * 1000 / engine.tree().capacity().max(1);
        let chess960 = engine.params().general().chess960();

        let pvs = (0..)
            .map_while(|pv_idx| engine.pv(pv_idx))
            .take(engine.params().general().multi_pv());

        for (pv_idx, pv) in pvs.enumerate() {
            let score = match pv.state().mate_in() {
                Some(moves) => format!("mate {moves}"),
                None => format!("cp {}", pv.score().cp()),
            };
            let moves: Vec<String> = pv.moves().iter().map(|mv| mv.to_string(chess960)).collect();

            println!(
                "info depth {depth} seldepth {max_depth} score {score} time {time} nodes {nodes} nps {nps} hashfull {hashfull} multipv {} pv {}",
                pv_idx + 1,
                moves.join(" ")
            )
        }
    }

    pub fn print_best_move<C: EngineConfig>(mv: Move, engine: &Engine<C>) {
        println!(
            "bestmove {}",
            mv.to_string(engine.params().general().chess960())
        )
    }

    fn should_report(&self, stats: &SearchStats) -> bool {
        stats.avg_depth() > self.last_avg_depth.load(Ordering::Relaxed)
            || stats.max_depth() > self.last_max_depth.load(Ordering::Relaxed)
            || (stats.iterations().is_multiple_of(INTERVAL_CHECK_THRESHOLD)
                && stats.elapsed_ms()
                    >= self.last_report_time.load(Ordering::Relaxed) + REPORT_INTERVAL)
    }

    fn report<C: EngineConfig>(&self, stats: &SearchStats, engine: &Engine<C>)
    where
        C::BestMove: BestMoveStrategy<C>,
        C::Node: HasVisits,
        C::Edge: HasScore + HasVisits,
    {
        self.last_avg_depth
            .store(stats.avg_depth(), Ordering::Relaxed);
        self.last_max_depth
            .store(stats.max_depth(), Ordering::Relaxed);
        self.last_report_time
            .store(stats.elapsed_ms(), Ordering::Relaxed);

        Self::print_report(stats, engine);
    }
}

impl<C: EngineConfig> Reporter<C> for UciReporter
where
    C::BestMove: BestMoveStrategy<C>,
    C::Node: HasVisits,
    C::Edge: HasScore + HasVisits,
{
    fn print(&self, msg: &str) {
        println!("{msg}")
    }

    fn search_started(&self, _limits: &SearchLimits, _engine: &Engine<C>) {
        self.last_avg_depth.store(0, Ordering::Relaxed);
        self.last_max_depth.store(0, Ordering::Relaxed);
        self.last_report_time.store(0, Ordering::Relaxed);
    }

    fn search_update(&self, stats: &SearchStats, engine: &Engine<C>) {
        if self.should_report(stats) {
            self.report(stats, engine);
        }
    }

    fn search_ended(&self, stats: &SearchStats, engine: &Engine<C>) {
        self.report(stats, engine);
    }

    fn best_move(&self, mv: Move, engine: &Engine<C>) {
        Self::print_best_move(mv, engine);
    }
}
