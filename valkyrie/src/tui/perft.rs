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

use crate::tui::{
    canvas::{Canvas, Rect},
    palette::{ERROR, GOLD, GOLD_LIGHT, MUTED, Rgb, STEEL, TEXT, WHITE},
    report::{self, Progress, group, speed, time},
    state::PerftRun,
    theme::Style,
};

const TITLE: &str = "P E R F T";
const MOVE_WIDTH: usize = 6;
const MIN_NODES_WIDTH: usize = 7;
const MIN_BRANCHING: usize = 30;
const CELL_GAP: usize = 4;

pub fn header(canvas: &mut Canvas, area: Rect, run: &PerftRun) {
    report::header(
        canvas,
        area,
        TITLE,
        &[
            ("depth", run.depth.to_string()),
            ("bulk", switch(run.bulk).to_string()),
            ("PEXT", switch(cfg!(target_feature = "bmi2")).to_string()),
        ],
    );
}

pub fn splits(canvas: &mut Canvas, area: Rect, run: &PerftRun, tone: Option<Rgb>) {
    let highlight = tone.is_some();
    let background = report::panel(canvas, area, "Splits", tone);
    let grid = Rect::new(
        area.x,
        area.y + 1,
        area.width,
        area.height.saturating_sub(1),
    );
    let cell = MOVE_WIDTH + nodes_width(run);
    let columns = columns(grid.width, run);
    let rows = run.moves.div_ceil(columns).clamp(1, grid.height.max(1));
    let capacity = rows * columns;
    let hidden = run.splits.len().saturating_sub(capacity);

    for (index, (mv, nodes)) in run.splits.iter().enumerate().take(capacity) {
        let x = grid.x + (index / rows) * (cell + CELL_GAP);
        let y = grid.y + index % rows;
        let latest = !run.done() && index + 1 == run.splits.len();

        if hidden > 0 && index + 1 == capacity {
            canvas.put(
                x,
                y,
                &format!("+{} more", hidden + 1),
                Style::new(MUTED).on(background),
            );
            break;
        }

        let (move_color, nodes_color) = match (highlight, latest) {
            (true, _) => (WHITE, WHITE),
            (false, true) => (GOLD_LIGHT, GOLD_LIGHT),
            (false, false) => (TEXT, STEEL),
        };
        let nodes = group(*nodes);

        canvas.put(x, y, mv, Style::new(move_color).on(background).bold());
        canvas.put(
            x + cell - nodes.len(),
            y,
            &nodes,
            Style::new(nodes_color).on(background),
        );
    }

    if run.splits.is_empty() && !highlight {
        canvas.put(
            grid.x,
            grid.y,
            "Waiting for the first move…",
            Style::new(MUTED),
        );
    }
}

pub fn summary(canvas: &mut Canvas, area: Rect, run: &PerftRun, tone: Option<Rgb>) {
    let done = run.splits.len();
    let elapsed = run.elapsed();
    let progress = Progress {
        label: "Moves",
        filled: if run.moves == 0 {
            1.0
        } else {
            done as f32 / run.moves as f32
        },
        value: format!("{done} / {}", run.moves),
        color: match run.outcome {
            Some(outcome) if outcome.stopped => ERROR,
            _ => GOLD,
        },
    };

    report::summary(
        canvas,
        area,
        tone,
        Some(progress),
        &[
            ("Nodes", group(run.nodes())),
            ("Time", time(elapsed)),
            ("Speed", speed(run.nodes(), elapsed)),
        ],
    );
}

pub fn results_text(run: &PerftRun) -> String {
    let splits: String = run
        .splits
        .iter()
        .map(|(mv, nodes)| format!("{mv}: {nodes}\n"))
        .collect();

    format!("{splits}\n{}", summary_text(run))
}

pub fn summary_text(run: &PerftRun) -> String {
    let elapsed = run.elapsed();

    format!(
        "Depth: {}\nBulk: {}\nPEXT: {}\nNodes: {}\nTime: {}\nSpeed: {}",
        run.depth,
        switch(run.bulk),
        switch(cfg!(target_feature = "bmi2")),
        run.nodes(),
        time(elapsed),
        speed(run.nodes(), elapsed)
    )
}

pub fn columns(width: usize, run: &PerftRun) -> usize {
    ((width + CELL_GAP) / (MOVE_WIDTH + nodes_width(run) + CELL_GAP)).max(1)
}

fn nodes_width(run: &PerftRun) -> usize {
    let branching = run.moves.max(MIN_BRANCHING) as f64;
    let estimate = branching.powi(i32::from(run.depth.saturating_sub(1))) as u64;

    run.splits
        .iter()
        .map(|(_, nodes)| *nodes)
        .chain(std::iter::once(estimate))
        .map(|nodes| group(nodes).len())
        .max()
        .unwrap_or(1)
        .max(MIN_NODES_WIDTH)
}

fn switch(enabled: bool) -> &'static str {
    if enabled { "on" } else { "off" }
}
