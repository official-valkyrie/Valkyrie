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

use valkyrie_engine::prelude::ReportScore;

use crate::tui::{
    canvas::{Canvas, Rect},
    graphics::WdlImage,
    palette::{
        BACKGROUND, BALANCED, GOLD, GOLD_LIGHT, HOVER, Hsl, LOSING, MUTED, Rgb, STEEL, TEXT, WHITE,
        WINNING,
    },
    report::{self, Progress, group, speed, time},
    state::{HistoryEntry, Region, SearchLine, SearchRun, State, Target},
    theme::Style,
};

const TITLE: &str = "S E A R C H";
const EVAL_SCALE: f32 = 400.0;
const DRAW_PEAK: f32 = 0.6;
const RANK_WIDTH: usize = 3;
const SCORE_WIDTH: usize = 6;
const TIME_WIDTH: usize = 7;
const COLUMN_GAP: usize = 2;
const SECTION_GAP: usize = 1;
pub const WDL_WIDTH: usize = 6;
const WDL_LABEL_OFFSET: usize = 2;
const LABEL_GAP: usize = 1;
const ELLIPSIS: &str = " …";
const MAIN_LINE_ROWS: usize = 3;
const MIN_HISTORY_ROWS: usize = 2;

pub fn header(canvas: &mut Canvas, area: Rect, hash: String) {
    report::header(canvas, area, TITLE, &[("Hash", hash)]);
}

pub fn results(
    canvas: &mut Canvas,
    area: Rect,
    run: &SearchRun,
    state: &State,
    regions: &mut Vec<Region>,
) {
    let width = area.width;
    let wanted = 1 + main_rows(run, width) + run.lines.len().saturating_sub(1);
    let history_room = area.height.saturating_sub(wanted + SECTION_GAP);
    let lines_rows = if history_room >= MIN_HISTORY_ROWS {
        wanted
    } else {
        area.height
    };

    lines_panel(
        canvas,
        Rect::new(area.x, area.y, width, lines_rows.max(1)),
        run,
        state,
        regions,
    );

    if history_room >= MIN_HISTORY_ROWS {
        history_panel(
            canvas,
            Rect::new(
                area.x,
                area.y + lines_rows + SECTION_GAP,
                width,
                history_room,
            ),
            run,
        );
    }
}

pub fn wdl_picture(area: Rect, run: &SearchRun) -> Option<WdlImage> {
    let score = run.lines.first()?.score;

    Some(WdlImage::new((area.x, area.y), area.height, wdl(score)))
}

pub fn wdl_column(canvas: &mut Canvas, area: Rect, run: &SearchRun, images: bool) {
    let Some(score) = run.lines.first().map(|line| line.score) else {
        return;
    };

    let shares = wdl(score);
    let colors = [WINNING, GOLD, LOSING];
    let height = area.height as f32;
    let mut start = 0.0;
    let mut rows = [0usize; 3];

    for (index, share) in shares.iter().enumerate() {
        let middle = height - (start + share / 2.0) * height;
        rows[index] = (middle.floor().max(0.0) as usize).min(area.height.saturating_sub(1));
        start += share;
    }

    rows[1] = rows[1].min(rows[0].saturating_sub(LABEL_GAP));
    rows[2] = rows[2].min(rows[1].saturating_sub(LABEL_GAP));

    if rows[2] == 0 {
        rows[1] = rows[1].max(LABEL_GAP);
        rows[0] = rows[0].max(rows[1] + LABEL_GAP);
    }

    for index in 0..3 {
        canvas.put(
            area.x + WDL_LABEL_OFFSET,
            area.y + rows[index],
            &format!("{:.0}%", shares[index] * 100.0),
            Style::new(colors[index]).bold(),
        );
    }

    if !images {
        report::column_meter(
            canvas,
            (area.x, area.y),
            area.height,
            &[
                (shares[0], colors[0]),
                (shares[1], colors[1]),
                (shares[2], colors[2]),
            ],
        );
    }
}

fn lines_panel(
    canvas: &mut Canvas,
    area: Rect,
    run: &SearchRun,
    state: &State,
    regions: &mut Vec<Region>,
) {
    report::panel(canvas, area, "Lines", None);

    let hovered = hovered(state);
    let heights: Vec<usize> = (0..run.lines.len())
        .map(|index| {
            if index == 0 {
                main_rows(run, area.width)
            } else {
                1
            }
        })
        .collect();
    let capacity = area.height.saturating_sub(1);
    let limit = capacity - usize::from(heights.iter().sum::<usize>() > capacity);
    let mut y = area.y + 1;
    let mut shown = 0;

    for (index, (line, rows)) in run.lines.iter().zip(&heights).enumerate() {
        let rows = *rows;

        if y + rows > area.y + 1 + limit {
            break;
        }

        shown += 1;

        let row = Rect::new(area.x, y, area.width, rows);
        let background = if hovered == Some(index) {
            canvas.fill(
                Rect::new(row.x.saturating_sub(1), row.y, row.width + 2, rows),
                Style::new(TEXT).on(HOVER),
            );
            HOVER
        } else {
            BACKGROUND
        };

        line_row(canvas, row, index, line, background);
        regions.push(Region {
            area: row,
            target: Target::Line {
                index,
                text: format!("{} {}", score_text(line.score), line.moves.join(" ")),
            },
        });
        y += rows;
    }

    if shown < run.lines.len() && y < area.bottom() {
        canvas.put(
            area.x + RANK_WIDTH,
            y,
            &format!("+{} more", run.lines.len() - shown),
            Style::new(MUTED),
        );
    }

    if run.lines.is_empty() {
        canvas.put(
            area.x,
            area.y + 1,
            "Waiting for the first report…",
            Style::new(MUTED),
        );
    }
}

fn line_row(canvas: &mut Canvas, row: Rect, index: usize, line: &SearchLine, background: Rgb) {
    let best = index == 0;
    let mut x = row.x;

    canvas.put(
        x,
        row.y,
        &(index + 1).to_string(),
        Style::new(if best { GOLD } else { MUTED }).on(background),
    );
    x += RANK_WIDTH;

    let score = score_text(line.score);
    canvas.put(
        x + SCORE_WIDTH.saturating_sub(score.len()),
        row.y,
        &score,
        Style::new(tone(expectation(line.score)))
            .on(background)
            .bold(),
    );
    x += SCORE_WIDTH + COLUMN_GAP;

    let width = (row.x + row.width).saturating_sub(x);

    moves(
        canvas,
        Rect::new(x, row.y, width, row.height),
        &line.moves,
        best,
        background,
    );
}

fn moves(canvas: &mut Canvas, area: Rect, line: &[String], best: bool, background: Rgb) {
    let (rows, hidden) = wrap(line, area.width, area.height);
    let last = rows.len().saturating_sub(1);

    for (offset, text) in rows.iter().enumerate() {
        let y = area.y + offset;
        let split = if offset == 0 {
            text.find(' ').unwrap_or(text.len())
        } else {
            0
        };
        let first = Style::new(if best { GOLD_LIGHT } else { WHITE })
            .on(background)
            .bold();
        let mut end = canvas.put(area.x, y, &text[..split], first);

        end = canvas.put(end, y, &text[split..], Style::new(STEEL).on(background));

        if offset == last && hidden > 0 {
            let end = canvas.put(end, y, ELLIPSIS, Style::new(MUTED).on(background));

            canvas.put(
                end + 1,
                y,
                &hidden_chip(hidden),
                Style::new(MUTED).on(background),
            );
        }
    }
}

fn history_panel(canvas: &mut Canvas, area: Rect, run: &SearchRun) {
    report::panel(canvas, area, "History", None);

    for (row, entry) in run
        .history
        .iter()
        .rev()
        .take(area.height.saturating_sub(1))
        .enumerate()
    {
        history_row(canvas, area, area.y + 1 + row, entry, row == 0);
    }
}

fn history_row(canvas: &mut Canvas, area: Rect, y: usize, entry: &HistoryEntry, current: bool) {
    let mut x = area.x;
    let stamp = time(entry.elapsed);
    let score = score_text(entry.score);

    canvas.put(
        x + TIME_WIDTH.saturating_sub(stamp.len()),
        y,
        &stamp,
        Style::new(if current { TEXT } else { MUTED }),
    );
    x += TIME_WIDTH + COLUMN_GAP;
    canvas.put(
        x + SCORE_WIDTH.saturating_sub(score.len()),
        y,
        &score,
        Style::new(tone(expectation(entry.score))),
    );
    x += SCORE_WIDTH + COLUMN_GAP;

    let width = (area.x + area.width).saturating_sub(x);

    moves(
        canvas,
        Rect::new(x, y, width, 1),
        &entry.moves,
        current,
        BACKGROUND,
    );
}

pub fn summary(canvas: &mut Canvas, area: Rect, run: &SearchRun, tone: Option<Rgb>) {
    let usage = run.hashfull as f32 / 1000.0;

    report::summary(
        canvas,
        area,
        tone,
        Some(Progress {
            label: "Tree",
            filled: usage,
            value: format!("{:.1}%", usage * 100.0),
            color: GOLD,
        }),
        &[
            ("Depth", format!("{} / {}", run.depth, run.seldepth)),
            ("Nodes", group(run.nodes)),
            ("Time", time(run.elapsed)),
            ("Speed", speed(run.nodes, run.elapsed)),
        ],
    );
}

pub fn summary_text(run: &SearchRun) -> String {
    let best = run.lines.first();

    format!(
        "Best move: {}\nScore: {}\nPV: {}\nDepth: {}\nMax depth: {}\nNodes: {}\nTime: {}\nSpeed: {}\nTree: {:.1}%",
        run.best.as_deref().unwrap_or("-"),
        best.map_or_else(|| "-".to_string(), |line| score_text(line.score)),
        best.map_or_else(String::new, |line| line.moves.join(" ")),
        run.depth,
        run.seldepth,
        run.nodes,
        time(run.elapsed),
        speed(run.nodes, run.elapsed),
        run.hashfull as f32 / 10.0
    )
}

pub fn score_text(score: ReportScore) -> String {
    match score.mate_in {
        Some(moves) if moves < 0 => format!("-M{}", moves.unsigned_abs()),
        Some(moves) => format!("M{moves}"),
        None => format!("{:+.2}", f64::from(score.cp) / 100.0),
    }
}

pub fn expectation(score: ReportScore) -> f32 {
    match score.mate_in {
        Some(moves) if moves > 0 => 1.0,
        Some(_) => 0.0,
        None => 1.0 / (1.0 + (-(score.cp as f32) / EVAL_SCALE).exp()),
    }
}

fn wdl(score: ReportScore) -> [f32; 3] {
    let expected = expectation(score);
    let balance = 2.0 * expected - 1.0;
    let draw = (DRAW_PEAK * (1.0 - balance * balance)).min(2.0 * expected.min(1.0 - expected));

    [expected - draw / 2.0, draw, 1.0 - expected - draw / 2.0]
}

pub fn tone(score: f32) -> Rgb {
    let t = score.clamp(0.0, 1.0);

    const EDGE_DEADZONE: f32 = 0.35;
    const DRAW_DEADZONE: f32 = 0.0075;

    if t <= EDGE_DEADZONE {
        return LOSING;
    }
    if t >= (1.0 - EDGE_DEADZONE) {
        return WINNING;
    }
    if ((0.5 - DRAW_DEADZONE)..=(0.5 + DRAW_DEADZONE)).contains(&t) {
        return BALANCED;
    }

    let (from, to, local) = if t < 0.5 {
        let local = (t - EDGE_DEADZONE) / ((0.5 - DRAW_DEADZONE) - EDGE_DEADZONE);
        (LOSING, BALANCED, local)
    } else {
        let local = (t - (0.5 + DRAW_DEADZONE)) / ((1.0 - EDGE_DEADZONE) - (0.5 + DRAW_DEADZONE));
        (BALANCED, WINNING, local)
    };

    Hsl::from_rgb(from)
        .lerp(Hsl::from_rgb(to), local.clamp(0.0, 1.0))
        .to_rgb()
}

pub fn hovered(state: &State) -> Option<usize> {
    match state.hovered_target() {
        Some(Target::Line { index, .. }) => Some(*index),
        _ => None,
    }
}

fn main_rows(run: &SearchRun, width: usize) -> usize {
    let room = width.saturating_sub(RANK_WIDTH + SCORE_WIDTH + COLUMN_GAP);

    run.lines.first().map_or(1, |line| {
        wrap(&line.moves, room, MAIN_LINE_ROWS).0.len().max(1)
    })
}

fn hidden_chip(hidden: usize) -> String {
    format!("+{hidden}")
}

fn wrap(moves: &[String], width: usize, rows: usize) -> (Vec<String>, usize) {
    let length = |line: &[&str]| {
        line.iter()
            .map(|mv| mv.chars().count() + 1)
            .sum::<usize>()
            .saturating_sub(1)
    };
    let mut done = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for (index, mv) in moves.iter().enumerate() {
        if !current.is_empty() && length(&current) + 1 + mv.chars().count() > width {
            if done.len() + 1 < rows {
                done.push(current.join(" "));
                current.clear();
            } else {
                let mut hidden = moves.len() - index;

                while !current.is_empty()
                    && length(&current) + ELLIPSIS.chars().count() + 1 + hidden_chip(hidden).len()
                        > width
                {
                    current.pop();
                    hidden += 1;
                }

                done.push(current.join(" "));
                return (done, hidden);
            }
        }

        current.push(mv);
    }

    if !current.is_empty() {
        done.push(current.join(" "));
    }

    (done, 0)
}
