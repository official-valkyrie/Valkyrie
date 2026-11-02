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

use valkyrie_engine::prelude::{OptionKind, OptionSpec};

use crate::tui::{
    canvas::{Canvas, Rect, fit},
    completion::{Catalog, CommandInfo, Suggestions},
    palette::{
        BACKGROUND, DARK_SQUARE, ERROR, GOLD, GOLD_LIGHT, HOVER, LIGHT_SQUARE, MUTED, NAVY, RULE,
        Rgb, STEEL, SURFACE, TEXT, WHITE,
    },
    state::{Edit, Position, Region, Screen, State, Target, Tone},
    theme::{ACCENT, DIM, FAILURE, PLAIN, Style},
};

pub const INPUT_HEIGHT: usize = 3;
pub const BOARD_WIDTH: usize = BOARD_LABEL_WIDTH + BOARD_SQUARE_WIDTH * 8;
pub const BOARD_ROWS: usize = 9;
pub const DETAIL_ROWS: usize = 5;
pub const FEN_LABEL_WIDTH: usize = FEN_LABEL.len();

pub const FEN_LABEL: &str = "FEN: ";

pub const BOARD_LABEL_WIDTH: usize = 2;
pub const BOARD_SQUARE_WIDTH: usize = 2;
const DETAIL_LABEL_WIDTH: usize = 12;
const FILES: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
const PIECE_GLYPHS: [char; 6] = ['♙', '♞', '♝', '♜', '♛', '♚'];
const APPEARANCE: [usize; 6] = [5, 3, 2, 4, 1, 0];

const PROMPT: &str = "› ";
const PLACEHOLDER: &str = "Type a command, or ? for help";
const HELP_TITLE: &str = "Commands";
const HELP_MAX_WIDTH: usize = 112;
const HELP_PADDING: usize = 2;
const HELP_COLUMN_GAP: usize = 3;
const HELP_TIP: &str = "Tab completes commands and their arguments as you type";
const HELP_CLOSE_KEY: &str = "any key";
const HELP_CLOSE_LABEL: &str = "close";
const HELP_DESCRIPTION_WIDTH: usize = 56;
const HELP_CHROME_ROWS: usize = 6;
const BACKDROP: f32 = 0.75;
const POPUP_LIMIT_WIDTH: usize = 96;
const IDLE_CHIPS: [(&str, &str); 3] = [("Enter", "run"), ("Tab", "complete"), ("↑↓", "history")];
const QUIT_LABEL: &str = "Quit";
const STOP_LABEL: &str = "Stop";
const CHIP_GAP: usize = 3;
const COPY_NOTICE: Duration = Duration::from_secs(2);
const QUIT_KEY: &str = "ctrl+c";
const EDIT_MIN_WIDTH: usize = 8;

pub fn heading(canvas: &mut Canvas, x: usize, y: usize, width: usize, title: &str) {
    canvas.heading(x, y, width, title, Style::new(GOLD).bold());
}

pub fn visible_options(catalog: &Catalog) -> impl Iterator<Item = &OptionSpec> {
    catalog
        .options
        .iter()
        .filter(|spec| !spec.tunable && spec.kind != OptionKind::Button)
}

pub fn settings(
    canvas: &mut Canvas,
    area: Rect,
    catalog: &Catalog,
    state: &State,
    regions: &mut Vec<Region>,
) {
    for (row, spec) in visible_options(catalog).take(area.height).enumerate() {
        let line = Rect::new(area.x, area.y + row, area.width, 1);
        let editing = state.editing.as_ref().filter(|edit| edit.name == spec.name);
        let active = editing.is_some() || state.hovers(line);
        let background = if active { HOVER } else { BACKGROUND };

        canvas.fill(
            Rect::new(line.x.saturating_sub(1), line.y, line.width + 2, 1),
            Style::new(TEXT).on(background),
        );
        canvas.put(
            line.x,
            line.y,
            &fit(&spec.name, line.width / 2),
            Style::new(if active { WHITE } else { STEEL }).on(background),
        );

        match editing {
            Some(edit) => edit_field(canvas, line, edit),
            None => value(canvas, line, spec, state, background),
        }

        regions.push(Region {
            area: Rect::new(line.x.saturating_sub(1), line.y, line.width + 2, 1),
            target: Target::Setting(spec.name.clone()),
        });
    }
}

fn value(canvas: &mut Canvas, line: Rect, spec: &OptionSpec, state: &State, background: Rgb) {
    let current = state.option_value(&spec.name, &spec.default);
    let changed = current != spec.default;

    let (text, style) = match spec.kind {
        OptionKind::Check if current.eq_ignore_ascii_case("true") => {
            ("On".to_string(), Style::new(GOLD_LIGHT))
        }
        OptionKind::Check => ("Off".to_string(), Style::new(MUTED)),
        _ if current.is_empty() => ("–".to_string(), Style::new(MUTED)),
        _ if changed => (current.to_string(), Style::new(GOLD_LIGHT).bold()),
        _ => (current.to_string(), Style::new(TEXT)),
    };

    let text = fit(&text, line.width / 2);
    canvas.put(
        line.x + line.width - text.chars().count(),
        line.y,
        &text,
        style.on(background),
    );
}

fn edit_field(canvas: &mut Canvas, line: Rect, edit: &Edit) {
    let width = (edit.value.chars().count() + 2)
        .max(EDIT_MIN_WIDTH)
        .min(line.width / 2);
    let x = line.x + line.width - width;
    let shown: String = edit
        .value
        .chars()
        .rev()
        .take(width - 2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    canvas.fill(Rect::new(x, line.y, width, 1), Style::new(WHITE).on(NAVY));

    let text_x = x + width - 1 - shown.chars().count();
    let text_style = if edit.fresh {
        Style::new(BACKGROUND).on(GOLD_LIGHT)
    } else {
        Style::new(WHITE).on(NAVY)
    };

    canvas.put(text_x, line.y, &shown, text_style);
    canvas.set(x + width - 1, line.y, ' ', Style::new(BACKGROUND).on(GOLD));
}

pub fn response(canvas: &mut Canvas, x: usize, y: usize, width: usize, state: &State) {
    let Some(response) = &state.response else {
        return;
    };

    let (marker, marker_style, text_style) = match response.tone {
        Tone::Pending if state.busy => ("●", ACCENT, Style::new(TEXT)),
        Tone::Pending => ("›", DIM, Style::new(MUTED)),
        Tone::Info => ("›", DIM, Style::new(STEEL)),
        Tone::Result => ("›", ACCENT, Style::new(GOLD_LIGHT).bold()),
        Tone::Error => ("×", FAILURE.bold(), Style::new(ERROR)),
    };

    canvas.put(x, y, marker, marker_style);

    let end = canvas.put(
        x + 2,
        y,
        &fit(&response.text, width.saturating_sub(2)),
        text_style,
    );

    if response.tone == Tone::Pending && state.busy {
        canvas.put(
            end,
            y,
            &fit("  ·  searching", (x + width).saturating_sub(end)),
            DIM,
        );
    }
}

pub fn input(canvas: &mut Canvas, area: Rect, state: &State, suggestions: &Suggestions) {
    let focused = state.editing.is_none();
    let inner = canvas.frame(area, "", Style::new(RULE), ACCENT);
    let field = Style::new(TEXT).on(SURFACE);
    let editor = &state.editor;

    canvas.fill(inner, field);

    if inner.width <= PROMPT.chars().count() + 2 || inner.height == 0 {
        return;
    }

    canvas.put(
        inner.x + 1,
        inner.y,
        PROMPT,
        Style::new(if focused { GOLD } else { MUTED })
            .on(SURFACE)
            .bold(),
    );

    let text_x = inner.x + 1 + PROMPT.chars().count();
    let room = inner.width.saturating_sub(PROMPT.chars().count() + 2);
    let buffer: Vec<char> = editor.text().chars().collect();
    let cursor = editor.cursor();
    let offset = (cursor + 1).saturating_sub(room);

    for (column, symbol) in buffer.iter().skip(offset).take(room).enumerate() {
        canvas.set(
            text_x + column,
            inner.y,
            *symbol,
            Style::new(WHITE).on(SURFACE),
        );
    }

    let typed: String = buffer
        .get(suggestions.start.min(buffer.len())..)
        .map(|rest| rest.iter().collect())
        .unwrap_or_default();

    let cursor_column = cursor - offset;
    let trailing = if buffer.is_empty() {
        PLACEHOLDER.to_string()
    } else if editor.at_end() {
        suggestions
            .ghost(&typed, editor.selected())
            .unwrap_or_default()
    } else {
        String::new()
    };

    if cursor_column < room {
        canvas.put(
            text_x + cursor_column,
            inner.y,
            &fit(&trailing, room - cursor_column),
            Style::new(MUTED).on(SURFACE),
        );

        if focused {
            let under = buffer
                .get(cursor)
                .copied()
                .or_else(|| trailing.chars().next())
                .unwrap_or(' ');

            canvas.set(
                text_x + cursor_column,
                inner.y,
                under,
                Style::new(BACKGROUND).on(GOLD),
            );
        }
    }
}

pub fn footer(
    canvas: &mut Canvas,
    area: Rect,
    catalog: &Catalog,
    state: &State,
    suggestions: &Suggestions,
    regions: &mut Vec<Region>,
) {
    let label = if state.busy { STOP_LABEL } else { QUIT_LABEL };
    let quit_width = chip_width(QUIT_KEY, label);
    let quit = Rect::new(area.x + area.width - quit_width, area.y, quit_width, 1);

    chip(canvas, quit.x, quit.y, QUIT_KEY, label, state.hovers(quit));
    regions.push(Region {
        area: quit,
        target: Target::Quit,
    });

    let limit = quit.x.saturating_sub(CHIP_GAP);
    let mut x = area.x;

    let text = |canvas: &mut Canvas, x: &mut usize, value: &str, style: Style| {
        *x = canvas.put(*x, area.y, &fit(value, limit.saturating_sub(*x)), style);
    };

    if let Some(copied) = state
        .copied
        .as_ref()
        .filter(|copied| copied.at.elapsed() < COPY_NOTICE)
    {
        text(
            canvas,
            &mut x,
            &copied.message,
            Style::new(GOLD_LIGHT).bold(),
        );
        return;
    }

    if let Some(edit) = &state.editing {
        for (key, label) in [("Enter", "apply"), ("Esc", "cancel")] {
            x = chip_within(canvas, x, area.y, key, label, limit);
        }
        text(canvas, &mut x, &range(catalog, &edit.name), DIM);
        return;
    }

    match state.hovered_target() {
        Some(Target::Setting(name)) => {
            let toggles = catalog
                .option(name)
                .is_some_and(|spec| matches!(spec.kind, OptionKind::Check | OptionKind::Combo));

            text(canvas, &mut x, "Click to ", DIM);
            text(
                canvas,
                &mut x,
                &format!("{} {name}", if toggles { "toggle" } else { "edit" }),
                Style::new(STEEL),
            );
            let range = range(catalog, name);

            if !range.is_empty() {
                text(canvas, &mut x, &format!("  ·  {range}"), DIM);
            }
        }
        Some(Target::Suggestion(_)) => {
            text(canvas, &mut x, "Click to ", DIM);
            text(canvas, &mut x, "insert this suggestion", Style::new(STEEL));
        }
        Some(Target::Copy { label, .. }) => {
            text(canvas, &mut x, "Click to ", DIM);
            text(
                canvas,
                &mut x,
                &format!("copy the {label}"),
                Style::new(STEEL),
            );
        }
        Some(Target::Quit) => {
            let action = if state.busy {
                "Stop the search"
            } else {
                "Exit Valkyrie"
            };
            text(canvas, &mut x, action, Style::new(STEEL));
        }
        None if state.screen == Screen::Perft => match &state.perft {
            Some(run) if run.done() => {
                x = chip_within(canvas, x, area.y, "any key", "back to the console", limit);
                text(canvas, &mut x, "Hover the results to copy them", DIM);
            }
            _ => text(canvas, &mut x, "Counting legal moves…", DIM),
        },
        None => match suggestions.usage {
            Some(usage) => {
                text(canvas, &mut x, "usage  ", DIM);
                text(canvas, &mut x, usage, Style::new(STEEL));
            }
            None => {
                for (key, label) in IDLE_CHIPS {
                    x = chip_within(canvas, x, area.y, key, label, limit);
                }
            }
        },
    }
}

fn range(catalog: &Catalog, name: &str) -> String {
    catalog
        .option(name)
        .and_then(|spec| spec.range.as_ref())
        .map_or(String::new(), |(min, max)| format!("range {min}–{max}"))
}

fn chip_width(key: &str, label: &str) -> usize {
    key.chars().count() + 2 + 1 + label.chars().count()
}

fn chip(canvas: &mut Canvas, x: usize, y: usize, key: &str, label: &str, hot: bool) -> usize {
    let (key_style, label_style) = if hot {
        (
            Style::new(BACKGROUND).on(GOLD).bold(),
            Style::new(WHITE).bold(),
        )
    } else {
        (Style::new(GOLD_LIGHT).on(SURFACE), Style::new(STEEL))
    };

    let end = canvas.put(x, y, &format!(" {key} "), key_style);
    canvas.put(end + 1, y, label, label_style)
}

fn chip_within(
    canvas: &mut Canvas,
    x: usize,
    y: usize,
    key: &str,
    label: &str,
    limit: usize,
) -> usize {
    if x + chip_width(key, label) > limit {
        return x;
    }

    chip(canvas, x, y, key, label, false) + CHIP_GAP
}

pub fn suggestions(
    canvas: &mut Canvas,
    input: Rect,
    state: &State,
    suggestions: &Suggestions,
    regions: &mut Vec<Region>,
) {
    let rows = suggestions.items.len().min(input.y.saturating_sub(2));

    if rows == 0 || state.editing.is_some() {
        return;
    }

    let width = input.width.min(POPUP_LIMIT_WIDTH);
    let area = Rect::new(input.x, input.y - rows - 2, width, rows + 2);
    let surface = Style::new(TEXT).on(SURFACE);

    canvas.fill(area, PLAIN);
    let inner = canvas.frame(area, "", Style::new(RULE), PLAIN);
    canvas.fill(inner, surface);

    let selected = state.editor.selected().min(suggestions.items.len() - 1);
    let label_width = suggestions
        .items
        .iter()
        .take(rows)
        .map(|item| item.label.chars().count())
        .max()
        .unwrap_or(0)
        .min(inner.width / 2);

    for (row, item) in suggestions.items.iter().take(rows).enumerate() {
        let line = Rect::new(inner.x, inner.y + row, inner.width, 1);
        let active = row == selected;
        let background = if active {
            NAVY
        } else if state.hovers(line) {
            HOVER
        } else {
            SURFACE
        };

        canvas.fill(line, surface.on(background));

        let label_style = if active {
            Style::new(WHITE).on(background).bold()
        } else {
            Style::new(GOLD_LIGHT).on(background)
        };

        canvas.put(
            line.x + 1,
            line.y,
            &fit(&item.label, label_width),
            label_style,
        );

        let detail_x = line.x + 1 + label_width + 2;

        if detail_x + 1 < line.x + line.width {
            canvas.put(
                detail_x,
                line.y,
                &fit(&item.detail, line.x + line.width - detail_x - 1),
                Style::new(if active { STEEL } else { MUTED }).on(background),
            );
        }

        regions.push(Region {
            area: line,
            target: Target::Suggestion(row),
        });
    }
}

pub fn help(canvas: &mut Canvas, catalog: &Catalog) {
    canvas.dim(BACKDROP);

    let mut groups: Vec<(&str, Vec<&CommandInfo>)> = Vec::new();

    for command in &catalog.commands {
        match groups
            .iter_mut()
            .find(|(category, _)| *category == command.category)
        {
            Some((_, commands)) => commands.push(command),
            None => groups.push((command.category, vec![command])),
        }
    }

    let aliases = |command: &CommandInfo| command.aliases.join(" ");
    let alias_width = catalog
        .commands
        .iter()
        .map(|command| aliases(command).chars().count())
        .max()
        .unwrap_or(0);
    let usage_width = catalog
        .commands
        .iter()
        .map(|command| command.usage.chars().count())
        .max()
        .unwrap_or(0);

    let width = (HELP_PADDING * 2
        + 2
        + usage_width
        + HELP_COLUMN_GAP
        + HELP_DESCRIPTION_WIDTH
        + HELP_COLUMN_GAP
        + alias_width)
        .min(HELP_MAX_WIDTH)
        .min(canvas.width().saturating_sub(4));
    let content_width = width.saturating_sub(HELP_PADDING * 2 + 2);
    let usage_width = usage_width.min(content_width * 2 / 5);
    let description_width = content_width
        .saturating_sub(usage_width + HELP_COLUMN_GAP * 2 + alias_width)
        .max(1);

    let room = canvas.height().saturating_sub(2 + HELP_CHROME_ROWS);
    let layouts = [
        (true, true, true),
        (false, true, true),
        (false, true, false),
        (false, false, false),
    ];
    let lines = |command: &CommandInfo, wrap: bool| {
        if wrap {
            wrap_words(command.description, description_width)
        } else {
            vec![fit(command.description, description_width)]
        }
    };
    let rows = |(wrap, headings, gaps): (bool, bool, bool)| {
        groups
            .iter()
            .map(|(_, commands)| {
                commands
                    .iter()
                    .map(|command| lines(command, wrap).len())
                    .sum::<usize>()
                    + usize::from(headings)
            })
            .sum::<usize>()
            + if gaps {
                groups.len().saturating_sub(1)
            } else {
                0
            }
    };
    let layout = layouts
        .into_iter()
        .find(|layout| rows(*layout) <= room)
        .unwrap_or((false, false, false));
    let (wrap, headings, gaps) = layout;

    let height = (rows(layout) + HELP_CHROME_ROWS).min(canvas.height().saturating_sub(2));
    let area = Rect::new(
        (canvas.width() - width) / 2,
        (canvas.height() - height) / 2,
        width,
        height,
    );
    let on = |color| Style::new(color).on(SURFACE);

    canvas.fill(area, on(TEXT));

    let inner = canvas.frame(area, HELP_TITLE, on(RULE), on(GOLD).bold());
    let content = Rect::new(
        inner.x + HELP_PADDING,
        inner.y + 1,
        content_width,
        inner.height.saturating_sub(4),
    );
    let description_x = content.x + usage_width + HELP_COLUMN_GAP;
    let alias_x = content.x + content.width - alias_width;
    let mut y = content.y;

    for (index, (category, commands)) in groups.iter().enumerate() {
        y += usize::from(index > 0 && gaps);

        if headings && y < content.bottom() {
            let end = canvas.put(content.x, y, category, on(GOLD).bold());

            for x in end + 1..content.x + content.width {
                canvas.set(x, y, '─', on(RULE));
            }

            y += 1;
        }

        for command in commands {
            if y >= content.bottom() {
                break;
            }

            let end = canvas.put(content.x, y, command.name, on(GOLD_LIGHT).bold());
            let arguments = command.usage.strip_prefix(command.name).unwrap_or("");

            syntax(
                canvas,
                end,
                y,
                (content.x + usage_width).saturating_sub(end),
                arguments,
            );

            let alias_text = aliases(command);
            canvas.put(
                alias_x + alias_width - alias_text.chars().count(),
                y,
                &alias_text,
                on(MUTED),
            );

            for line in lines(command, wrap) {
                if y >= content.bottom() {
                    break;
                }

                canvas.put(description_x, y, &line, on(STEEL));
                y += 1;
            }
        }
    }

    let footer_y = inner.bottom() - 2;
    let key = format!(" {HELP_CLOSE_KEY} ");
    let label_x = content.x + content.width - HELP_CLOSE_LABEL.len();
    let key_x = label_x - 1 - key.chars().count();
    let tip_room = key_x.saturating_sub(content.x + HELP_COLUMN_GAP);

    if tip_room >= HELP_TIP.len() {
        canvas.put(content.x, footer_y, HELP_TIP, on(MUTED));
    }

    canvas.put(key_x, footer_y, &key, Style::new(GOLD_LIGHT).on(NAVY));
    canvas.put(label_x, footer_y, HELP_CLOSE_LABEL, on(STEEL));
}

fn syntax(canvas: &mut Canvas, x: usize, y: usize, width: usize, text: &str) {
    let mut placeholder = false;

    for (column, symbol) in fit(text, width).chars().enumerate() {
        placeholder = match symbol {
            '<' => true,
            '>' => {
                canvas.set(x + column, y, symbol, Style::new(MUTED).on(SURFACE));
                false
            }
            _ => placeholder,
        };

        let color = if placeholder || "[]|.…<>".contains(symbol) {
            MUTED
        } else {
            STEEL
        };

        if symbol != '>' {
            canvas.set(x + column, y, symbol, Style::new(color).on(SURFACE));
        }
    }
}

fn wrap_words(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();

    for word in text.split_whitespace() {
        if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > width {
            lines.push(std::mem::take(&mut line));
        }

        if !line.is_empty() {
            line.push(' ');
        }

        line.push_str(word);
    }

    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }

    lines.into_iter().map(|line| fit(&line, width)).collect()
}

pub fn placement_order(position: &Position) -> [[usize; 8]; 8] {
    let flipped = position.black_to_move();
    let orient = |index: usize| if flipped { 7 - index } else { index };
    let distance = |index: usize| (2 * index).abs_diff(7).pow(2);
    let mut squares: Vec<_> = (0..8)
        .flat_map(|shown_row| (0..8).map(move |shown_file| (shown_row, shown_file)))
        .filter_map(|(shown_row, shown_file)| {
            position.squares[orient(shown_row)][orient(shown_file)].map(|piece| {
                (
                    APPEARANCE[piece.piece],
                    !piece.white,
                    distance(shown_row) + distance(shown_file),
                    shown_file,
                    7 - shown_row,
                )
            })
        })
        .collect();
    let mut order = [[usize::MAX; 8]; 8];

    squares.sort_unstable();

    for (slot, (.., shown_file, from_bottom)) in squares.into_iter().enumerate() {
        order[7 - from_bottom][shown_file] = slot;
    }

    order
}

pub fn board(
    canvas: &mut Canvas,
    x: usize,
    y: usize,
    position: &Position,
    fade: f32,
    glyphs: Option<&dyn Fn(usize) -> f32>,
) {
    let flipped = position.black_to_move();
    let orient = |index: usize| if flipped { 7 - index } else { index };
    let order = placement_order(position);

    for (shown_row, slots) in order.iter().enumerate() {
        let row = orient(shown_row);

        canvas.put(x, y + shown_row, &(8 - row).to_string(), DIM);

        for (shown_file, slot) in slots.iter().enumerate() {
            let file = orient(shown_file);
            let square = if (row + file).is_multiple_of(2) {
                LIGHT_SQUARE
            } else {
                DARK_SQUARE
            };
            let column = x + BOARD_LABEL_WIDTH + shown_file * BOARD_SQUARE_WIDTH;
            let (glyph, piece) = match (position.squares[row][file], glyphs) {
                (Some(placed), Some(opacity)) => (
                    PIECE_GLYPHS[placed.piece],
                    square.lerp(if placed.white { WHITE } else { GOLD }, opacity(*slot)),
                ),
                _ => (' ', TEXT),
            };

            canvas.set(column, y + shown_row, glyph, Style::new(piece).on(square));
            canvas.set(column + 1, y + shown_row, ' ', Style::new(piece).on(square));
        }

        canvas.set(
            x + BOARD_LABEL_WIDTH + shown_row * BOARD_SQUARE_WIDTH,
            y + 8,
            FILES[orient(shown_row)],
            DIM,
        );
    }

    if fade < 1.0 {
        for row in y..y + BOARD_ROWS {
            for column in x..x + BOARD_WIDTH {
                canvas.fade(column, row, fade);
            }
        }
    }
}

pub fn position_details(
    canvas: &mut Canvas,
    x: usize,
    y: usize,
    width: usize,
    position: &Position,
) {
    let (turn, turn_color) = if position.black_to_move() {
        ("Black", GOLD)
    } else {
        ("White", WHITE)
    };

    let rows = [
        ("Turn", turn, Style::new(turn_color).bold()),
        ("Castling", position.field(2), Style::new(TEXT)),
        ("En passant", position.field(3), Style::new(TEXT)),
        ("Halfmove", position.field(4), Style::new(TEXT)),
        ("Move", position.field(5), Style::new(TEXT)),
    ];

    for (row, (label, value, style)) in rows.into_iter().enumerate() {
        canvas.put(x, y + row, &fit(label, width), Style::new(STEEL));
        canvas.put(
            x + DETAIL_LABEL_WIDTH,
            y + row,
            &fit(value, width.saturating_sub(DETAIL_LABEL_WIDTH)),
            style,
        );
    }
}

pub fn fen_lines(position: &Position, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut rest = position.fen.as_str();

    while rest.len() > width && width > 0 {
        let head = &rest[..width];
        let (end, next) = match (head.rfind(' '), head.rfind('/')) {
            (Some(space), _) => (space, space + 1),
            (None, Some(slash)) => (slash + 1, slash + 1),
            (None, None) => (width, width),
        };

        lines.push(rest[..end].to_string());
        rest = &rest[next..];
    }

    lines.push(rest.to_string());
    lines
}

pub fn copyable(
    canvas: &mut Canvas,
    x: usize,
    y: usize,
    lines: &[String],
    style: Style,
    state: &State,
) -> Rect {
    let area = Rect::new(
        x,
        y,
        lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0),
        lines.len(),
    );

    let style = if state.hovers(area) {
        let highlight = Style::new(WHITE).on(HOVER);

        canvas.fill(
            Rect::new(x.saturating_sub(1), y, area.width + 2, area.height),
            highlight,
        );
        highlight
    } else {
        style
    };

    for (row, line) in lines.iter().enumerate() {
        canvas.put(x, y + row, line, style);
    }

    area
}
