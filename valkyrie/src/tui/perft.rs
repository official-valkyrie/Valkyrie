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

use std::time::Duration;

use crate::tui::{
    animation,
    canvas::{Canvas, Rect, fit},
    palette::{BACKGROUND, ERROR, GOLD, GOLD_LIGHT, HOVER, MUTED, RULE, Rgb, STEEL, TEXT, WHITE},
    state::{PerftRun, State, Target},
    theme::Style,
};

pub const RESULTS: &str = "results";
pub const SUMMARY: &str = "summary";
pub const SUMMARY_ROWS: usize = 5;

const TITLE: &str = "P E R F T";
const ANNOTATION_GAP: usize = 3;
const COPY_FLASH: Duration = Duration::from_millis(450);
const FLASH_STRENGTH: f32 = 0.32;
const FACT_SEPARATOR: &str = "·";
const MOVE_WIDTH: usize = 6;
const MIN_NODES_WIDTH: usize = 7;
const MIN_BRANCHING: usize = 30;
const CELL_GAP: usize = 4;
const COUNTER_GAP: usize = 3;
const STATS: usize = 3;

pub fn header(canvas: &mut Canvas, area: Rect, run: &PerftRun) {
    let facts = [
        ("depth", run.depth.to_string()),
        ("bulk", switch(run.bulk).to_string()),
        ("PEXT", switch(cfg!(target_feature = "bmi2")).to_string()),
    ];
    let mut x = canvas.put(area.x, area.y, TITLE, Style::new(GOLD).bold());

    for (index, (label, value)) in facts.iter().enumerate() {
        let separator = if index == 0 { "" } else { FACT_SEPARATOR };

        x = canvas.put(x + ANNOTATION_GAP, area.y, separator, Style::new(MUTED));
        x = canvas.put(
            x + usize::from(index > 0) * ANNOTATION_GAP,
            area.y,
            label,
            Style::new(MUTED),
        );
        x = canvas.put(x + 1, area.y, value, Style::new(TEXT));
    }
}

pub fn splits(canvas: &mut Canvas, area: Rect, run: &PerftRun, tone: Option<Rgb>) {
    let highlight = tone.is_some();
    let background = surface(canvas, area, tone);

    heading(canvas, area, "Splits", background);

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
    let highlight = tone.is_some();
    let background = surface(canvas, area, tone);
    let label = Style::new(if highlight { WHITE } else { STEEL }).on(background);
    let value = Style::new(WHITE).on(background).bold();

    heading(canvas, area, "Summary", background);

    let done = run.splits.len();
    let counter = format!("{done} / {}", run.moves);
    let bar_width = area.width.saturating_sub(counter.len() + COUNTER_GAP);
    let filled = if run.moves == 0 {
        bar_width
    } else {
        done * bar_width / run.moves
    };
    let bar_color = match run.outcome {
        Some(outcome) if outcome.stopped => ERROR,
        _ => GOLD,
    };

    for column in 0..bar_width {
        let (symbol, color) = if column < filled {
            ('━', bar_color)
        } else {
            ('─', RULE)
        };

        canvas.set(
            area.x + column,
            area.y + 1,
            symbol,
            Style::new(color).on(background),
        );
    }

    canvas.put(
        area.x + area.width - counter.len(),
        area.y + 1,
        &counter,
        Style::new(if highlight { WHITE } else { MUTED }).on(background),
    );

    let elapsed = run.elapsed();
    let stat_width = area.width / STATS;

    for (index, (name, text)) in [
        ("Nodes", group(run.nodes())),
        ("Time", time(elapsed)),
        ("Speed", speed(run.nodes(), elapsed)),
    ]
    .into_iter()
    .enumerate()
    {
        let x = area.x + index * stat_width;

        canvas.put(x, area.y + 3, name, label);
        canvas.put(
            x,
            area.y + 4,
            &fit(&text, stat_width.saturating_sub(1)),
            value,
        );
    }
}

pub fn tones(state: &State) -> (Option<Rgb>, Option<Rgb>) {
    let hovered = match state.hovered_target() {
        Some(Target::Copy { label, .. }) => Some(*label),
        _ => None,
    };
    let flash = state.copied.as_ref().and_then(|copied| {
        let left = 1.0 - copied.at.elapsed().as_secs_f32() / COPY_FLASH.as_secs_f32();
        (left > 0.0).then_some((copied.label.as_str(), left))
    });
    let tone = |labels: &[&str]| match flash {
        Some((label, left)) if labels.contains(&label) => {
            Some(HOVER.lerp(GOLD, FLASH_STRENGTH * animation::smooth(left)))
        }
        _ if hovered.is_some_and(|label| labels.contains(&label)) => Some(HOVER),
        _ => None,
    };
    (tone(&[RESULTS]), tone(&[RESULTS, SUMMARY]))
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

fn surface(canvas: &mut Canvas, area: Rect, tone: Option<Rgb>) -> Rgb {
    let Some(tone) = tone else {
        return BACKGROUND;
    };

    canvas.fill(
        Rect::new(
            area.x.saturating_sub(1),
            area.y,
            area.width + 2,
            area.height,
        ),
        Style::new(TEXT).on(tone),
    );
    tone
}

fn heading(canvas: &mut Canvas, area: Rect, title: &str, background: Rgb) {
    let end = canvas.put(
        area.x,
        area.y,
        title,
        Style::new(if background == BACKGROUND {
            GOLD
        } else {
            WHITE
        })
        .on(background)
        .bold(),
    );

    for x in end + 1..area.x + area.width {
        canvas.set(x, area.y, '─', Style::new(RULE).on(background));
    }
}

pub fn group(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::new();

    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }

        grouped.push(digit);
    }

    grouped
}

fn time(elapsed: Duration) -> String {
    let milliseconds = elapsed.as_millis();

    if milliseconds < 1000 {
        format!("{milliseconds} ms")
    } else {
        format!("{:.2} s", elapsed.as_secs_f64())
    }
}

fn speed(nodes: u64, elapsed: Duration) -> String {
    let rate = nodes as f64 / elapsed.as_secs_f64().max(0.001);

    match rate {
        rate if rate >= 1e9 => format!("{:.2} Gnps", rate / 1e9),
        rate if rate >= 1e6 => format!("{:.1} Mnps", rate / 1e6),
        rate if rate >= 1e3 => format!("{:.1} knps", rate / 1e3),
        rate => format!("{rate:.0} nps"),
    }
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
