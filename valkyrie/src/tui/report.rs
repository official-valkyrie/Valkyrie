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
    palette::{BACKGROUND, GOLD, HOVER, MUTED, RULE, Rgb, STEEL, TEXT, WHITE},
    state::{State, Target},
    theme::Style,
};

pub const RESULTS: &str = "results";
pub const SUMMARY: &str = "summary";
pub const SUMMARY_ROWS: usize = 4;

const ANNOTATION_GAP: usize = 3;
const COPY_FLASH: Duration = Duration::from_millis(450);
const FLASH_STRENGTH: f32 = 0.32;
const FACT_SEPARATOR: &str = "·";
const BAR_GAP: usize = 2;

pub struct Progress {
    pub label: &'static str,
    pub filled: f32,
    pub value: String,
    pub color: Rgb,
}

pub fn header(canvas: &mut Canvas, area: Rect, title: &str, facts: &[(&str, String)]) {
    let mut x = canvas.put(area.x, area.y, title, Style::new(GOLD).bold());

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

fn surface(canvas: &mut Canvas, area: Rect, tone: Option<Rgb>) -> Rgb {
    let background = tone.unwrap_or(BACKGROUND);

    if tone.is_some() {
        canvas.fill(
            Rect::new(
                area.x.saturating_sub(1),
                area.y,
                area.width + 2,
                area.height,
            ),
            Style::new(TEXT).on(background),
        );
    }

    background
}

pub fn panel(canvas: &mut Canvas, area: Rect, title: &str, tone: Option<Rgb>) -> Rgb {
    let background = surface(canvas, area, tone);
    let end = canvas.put(
        area.x,
        area.y,
        title,
        Style::new(if tone.is_some() { WHITE } else { GOLD })
            .on(background)
            .bold(),
    );

    for x in end + 1..area.x + area.width {
        canvas.set(x, area.y, '─', Style::new(RULE).on(background));
    }

    background
}

pub fn summary(
    canvas: &mut Canvas,
    area: Rect,
    tone: Option<Rgb>,
    progress: Option<Progress>,
    stats: &[(&str, String)],
) {
    let highlight = tone.is_some();
    let background = surface(canvas, area, tone);
    let label = Style::new(if highlight { WHITE } else { STEEL }).on(background);
    let value = Style::new(WHITE).on(background).bold();
    if let Some(progress) = progress {
        let bar_x = area.x + progress.label.len() + BAR_GAP;
        let bar_end = (area.x + area.width).saturating_sub(progress.value.len() + BAR_GAP);
        let width = bar_end.saturating_sub(bar_x);

        canvas.put(area.x, area.y, progress.label, label);
        meter(
            canvas,
            (bar_x, area.y),
            width,
            &[(progress.filled.clamp(0.0, 1.0), progress.color)],
            background,
        );

        canvas.put(
            area.x + area.width - progress.value.len(),
            area.y,
            &progress.value,
            Style::new(if highlight { WHITE } else { TEXT })
                .on(background)
                .bold(),
        );
    }

    let stat_width = area.width / stats.len().max(1);

    for (index, (name, text)) in stats.iter().enumerate() {
        let x = area.x + index * stat_width;

        canvas.put(x, area.y + 2, name, label);
        canvas.put(
            x,
            area.y + 3,
            &fit(text, stat_width.saturating_sub(1)),
            value,
        );
    }
}

pub fn meter(
    canvas: &mut Canvas,
    (x, y): (usize, usize),
    width: usize,
    segments: &[(f32, Rgb)],
    background: Rgb,
) {
    let mut spans = Vec::new();
    let mut start = 0.0;

    for (share, color) in segments {
        let end = start + share.max(0.0) * width as f32;
        spans.push((start, end, *color));
        start = end;
    }

    for column in 0..width {
        let (left, right) = (column as f32, column as f32 + 1.0);
        let mut covered = 0.0;
        let mut mix = [0.0f32; 3];

        for (start, end, color) in &spans {
            let overlap = (end.min(right) - start.max(left)).max(0.0);

            covered += overlap;
            mix[0] += f32::from(color.0) * overlap;
            mix[1] += f32::from(color.1) * overlap;
            mix[2] += f32::from(color.2) * overlap;
        }

        let (symbol, color) = if covered <= f32::EPSILON {
            ('─', RULE)
        } else {
            let blend = Rgb(
                (mix[0] / covered) as u8,
                (mix[1] / covered) as u8,
                (mix[2] / covered) as u8,
            );
            ('━', RULE.lerp(blend, covered.min(1.0)))
        };

        canvas.set(x + column, y, symbol, Style::new(color).on(background));
    }
}

pub fn column_meter(
    canvas: &mut Canvas,
    (x, y): (usize, usize),
    height: usize,
    segments: &[(f32, Rgb)],
) {
    let mut spans = Vec::new();
    let mut start = 0.0;

    for (share, color) in segments {
        let end = start + share.max(0.0) * height as f32;
        spans.push((start, end, *color));
        start = end;
    }

    for row in 0..height {
        let (low, high) = (row as f32, row as f32 + 1.0);
        let color = spans
            .iter()
            .max_by(|first, second| {
                let overlap = |(start, end, _): &&(f32, f32, Rgb)| end.min(high) - start.max(low);
                overlap(first).total_cmp(&overlap(second))
            })
            .map_or(RULE, |(_, _, color)| *color);

        canvas.set(x, y + height - 1 - row, '┃', Style::new(color));
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

pub fn time(elapsed: Duration) -> String {
    let milliseconds = elapsed.as_millis();

    if milliseconds < 1000 {
        format!("{milliseconds} ms")
    } else {
        format!("{:.2} s", elapsed.as_secs_f64())
    }
}

pub fn speed(nodes: u64, elapsed: Duration) -> String {
    let rate = nodes as f64 / elapsed.as_secs_f64().max(0.001);

    match rate {
        rate if rate >= 1e9 => format!("{:.2} Gnps", rate / 1e9),
        rate if rate >= 1e6 => format!("{:.1} Mnps", rate / 1e6),
        rate if rate >= 1e3 => format!("{:.1} knps", rate / 1e3),
        rate => format!("{rate:.0} nps"),
    }
}
