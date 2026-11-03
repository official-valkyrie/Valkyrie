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
use valkyrie_engine::prelude::*;

const REPORT_INTERVAL: u64 = 1000;
const INTERVAL_CHECK_THRESHOLD: u64 = 128;
const PERFT_SEPARATOR: &str = "-----------------------------------------------------------";

#[derive(Debug, Default)]
pub struct SimpleView {
    last_avg_depth: AtomicU64,
    last_max_depth: AtomicU64,
    last_report_time: AtomicU64,
}

impl SimpleView {
    fn print_report(report: &SearchReport) {
        for (pv_idx, line) in report.lines.iter().enumerate() {
            let score = match line.score.mate_in {
                Some(moves) => format!("mate {moves}"),
                None => format!("cp {}", line.score.cp),
            };
            let moves: Vec<String> = line
                .moves
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
}

impl View for SimpleView {
    fn start(&self) {
        println!("{}", env!("ENGINE_NAME"));
        println!("Type \"help\" to see what it can do");
    }

    fn message(&self, _level: MessageLevel, text: &str) {
        println!("{text}")
    }

    fn clear(&self) {
        let (program, args): (&str, &[&str]) = if cfg!(target_os = "windows") {
            ("cmd", &["/c", "cls"])
        } else {
            ("clear", &[])
        };

        let _ = std::process::Command::new(program)
            .args(args)
            .spawn()
            .and_then(|mut child| child.wait());
    }

    fn identify(&self, name: &str, author: &str) {
        println!("id name {name}");
        println!("id author {author}");
    }

    fn option_listed(&self, spec: &OptionSpec) {
        let mut line = format!("option name {} type {}", spec.name, spec.kind.uci_name());

        if spec.kind != OptionKind::Button {
            let default = if spec.default.is_empty() {
                "<empty>"
            } else {
                &spec.default
            };

            line += &format!(" default {default}");
        }

        if let Some((min, max)) = &spec.range
            && (spec.kind == OptionKind::Spin || spec.tunable)
        {
            line += &format!(" min {min} max {max}");
        }

        for variant in spec.variants {
            line += &format!(" var {variant}");
        }

        println!("{line}")
    }

    fn uci_ok(&self) {
        println!("uciok")
    }

    fn ready_ok(&self) {
        println!("readyok")
    }

    fn board(&self, board: &ChessBoard) {
        println!("{board}")
    }

    fn position_changed(&self, _board: &ChessBoard) {
        println!("Position set successfully")
    }

    fn option_changed(&self, name: &str, value: &str) {
        println!("Option {name} has been set to {value}")
    }

    fn button_pressed(&self, name: &str) {
        println!("Option {name} has been triggered")
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

    fn perft_move(&self, mv: Move, nodes: u64, chess960: bool) {
        println!("  {} - {nodes}", mv.to_string(chess960))
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

    fn perft_stopped(&self, nodes: u64, elapsed: Duration) {
        println!("\n{PERFT_SEPARATOR}");
        println!(
            "  Perft stopped! {nodes} nodes counted in {}ms",
            elapsed.as_millis()
        );
        println!("{PERFT_SEPARATOR}\n");
    }

    fn bench_finished(&self, nodes: u64, elapsed: Duration) {
        let milliseconds = elapsed.as_millis().max(1);

        println!(
            "Bench: {nodes} nodes {} nps",
            u128::from(nodes) * 1000 / milliseconds
        );
    }

    fn search_started(&self, _limits: &SearchLimits) {
        self.last_avg_depth.store(0, Ordering::Relaxed);
        self.last_max_depth.store(0, Ordering::Relaxed);
        self.last_report_time.store(0, Ordering::Relaxed);
    }

    fn wants_report(&self, stats: &SearchStats) -> bool {
        let due = stats.avg_depth() > self.last_avg_depth.load(Ordering::Relaxed)
            || stats.max_depth() > self.last_max_depth.load(Ordering::Relaxed)
            || (stats.iterations().is_multiple_of(INTERVAL_CHECK_THRESHOLD)
                && stats.elapsed_ms()
                    >= self.last_report_time.load(Ordering::Relaxed) + REPORT_INTERVAL);

        if due {
            self.last_avg_depth
                .store(stats.avg_depth(), Ordering::Relaxed);
            self.last_max_depth
                .store(stats.max_depth(), Ordering::Relaxed);
            self.last_report_time
                .store(stats.elapsed_ms(), Ordering::Relaxed);
        }

        due
    }

    fn report(&self, report: &SearchReport) {
        Self::print_report(report);
    }

    fn search_ended(&self, report: &SearchReport) {
        Self::print_report(report);
    }

    fn best_move(&self, mv: Option<Move>, chess960: bool) {
        match mv {
            Some(mv) => println!("bestmove {}", mv.to_string(chess960)),
            None => println!("bestmove 0000"),
        }
    }
}
