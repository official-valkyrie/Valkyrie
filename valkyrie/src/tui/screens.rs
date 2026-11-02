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
    animation,
    canvas::{Canvas, Rect, fit},
    completion::{Catalog, Suggestions},
    logo::Logo,
    palette::{GOLD, STEEL},
    state::{Region, State, Target},
    theme::{ACCENT, DIM, Style},
    widgets::{self, BOARD_ROWS, BOARD_WIDTH, FEN_LABEL, FEN_LABEL_WIDTH, INPUT_HEIGHT},
};

const MIN_WIDTH: usize = 44;
const MIN_HEIGHT: usize = 12;
const MARGIN: usize = 2;
const LOGO_GAP: usize = 5;
const PANEL_MIN_WIDTH: usize = 38;
const PANEL_MAX_WIDTH: usize = 46;
const SOLO_PANEL_WIDTH: usize = 60;
const SECTION_GAP: usize = 1;
const VERSION_GAP: usize = 3;
const TITLE_GAP: usize = 2;
const FEN_GAP: usize = 1;
const RESPONSE_ROWS: usize = 1;
const BOARD_GAP: usize = 4;
const DETAILS_MIN_WIDTH: usize = 18;
const TITLE: &str = "V A L K Y R I E";

pub struct Frame<'a> {
    pub state: &'a State,
    pub catalog: &'a Catalog,
    pub suggestions: &'a Suggestions,
    pub progress: Option<f32>,
}

pub fn draw(canvas: &mut Canvas, frame: &Frame<'_>) -> Vec<Region> {
    let (width, height) = (canvas.width(), canvas.height());
    let mut regions = Vec::new();

    if width < MIN_WIDTH || height < MIN_HEIGHT {
        let message = "Window too small";
        canvas.put(
            width.saturating_sub(message.len()) / 2,
            height / 2,
            message,
            ACCENT,
        );
        return regions;
    }

    let content_height = height - INPUT_HEIGHT - RESPONSE_ROWS - 1;
    let available = width - MARGIN * 2;
    let logo = Logo::get();
    let panel_width = available
        .saturating_sub(logo.width + LOGO_GAP)
        .min(PANEL_MAX_WIDTH);
    let mut animated = Vec::new();

    if panel_width >= PANEL_MIN_WIDTH && content_height >= logo.height {
        let x = MARGIN + (available - logo.width - LOGO_GAP - panel_width) / 2;
        let y = (content_height - logo.height) / 2;

        logo.draw(canvas, x, y, frame.progress);
        animated.push(Rect::new(x, y, logo.width, logo.height));
        animated.extend(panel(
            canvas,
            frame,
            Rect::new(x + logo.width + LOGO_GAP, y, panel_width, logo.height),
            &mut regions,
        ));
    } else {
        let panel_width = available.min(SOLO_PANEL_WIDTH);

        animated.extend(panel(
            canvas,
            frame,
            Rect::new(
                MARGIN + (available - panel_width) / 2,
                0,
                panel_width,
                content_height,
            ),
            &mut regions,
        ));
    }

    let input = Rect::new(
        MARGIN,
        content_height + RESPONSE_ROWS,
        width - MARGIN * 2,
        INPUT_HEIGHT,
    );

    widgets::response(
        canvas,
        input.x + 1,
        content_height,
        input.width - 2,
        frame.state,
    );
    widgets::input(canvas, input, frame.state, frame.suggestions);
    widgets::footer(
        canvas,
        Rect::new(input.x + 1, height - 1, input.width - 2, 1),
        frame.catalog,
        frame.state,
        frame.suggestions,
        &mut regions,
    );
    widgets::suggestions(canvas, input, frame.state, frame.suggestions, &mut regions);

    if frame.state.help_open {
        widgets::help(canvas, frame.catalog);
    }

    if let Some(progress) = frame.progress {
        animation::reveal_rows(canvas, progress, &animated);
    }

    regions
}

fn panel(
    canvas: &mut Canvas,
    frame: &Frame<'_>,
    area: Rect,
    regions: &mut Vec<Region>,
) -> Option<Rect> {
    let (x, width) = (area.x, area.width);
    let position = frame.state.position.as_ref();
    let fen = position.map_or(Vec::new(), |position| {
        widgets::fen_lines(position, width.saturating_sub(FEN_LABEL_WIDTH))
    });
    let settings = widgets::visible_options(frame.catalog).count();

    let title_rows = 1 + TITLE_GAP;
    let board_rows = |fen_rows: usize| BOARD_ROWS + FEN_GAP + fen_rows + SECTION_GAP;
    let total = |board: Option<usize>| title_rows + board.map_or(0, board_rows) + 1 + settings;

    let board = position.map(|_| fen.len());
    let board = [board, board.map(|_| 1), None]
        .into_iter()
        .find(|board| total(*board) <= area.height)
        .flatten();

    let mut y = area.y + area.height.saturating_sub(total(board)) / 2;
    let end = canvas.put(x, y, TITLE, Style::new(GOLD).bold());
    let version = version(&frame.catalog.engine_name);
    let room = (x + width).saturating_sub(end + VERSION_GAP);

    if room > 0 {
        let area = widgets::copyable(
            canvas,
            end + VERSION_GAP,
            y,
            &[fit(version, room)],
            DIM,
            frame.state,
        );
        regions.push(Region {
            area,
            target: Target::Copy {
                label: "version",
                text: version.to_string(),
            },
        });
    }

    y += title_rows;

    let mut board_area = None;

    if let (Some(fen_rows), Some(position)) = (board, position) {
        let start = animation::board_start(y, canvas.height());
        let pieces = frame.progress.map_or(usize::MAX, |progress| {
            animation::pieces_placed(progress, start, position.piece_count())
        });

        if frame.progress.is_none_or(|progress| progress >= start) {
            widgets::board(canvas, x, y, position, pieces);
        }

        board_area = Some(Rect::new(x, y, BOARD_WIDTH, BOARD_ROWS));

        if width >= BOARD_WIDTH + BOARD_GAP + DETAILS_MIN_WIDTH {
            widgets::position_details(
                canvas,
                x + BOARD_WIDTH + BOARD_GAP,
                y,
                width - BOARD_WIDTH - BOARD_GAP,
                position,
            );
        }

        let lines = if fen_rows == fen.len() {
            fen.clone()
        } else {
            vec![fit(&position.fen, width.saturating_sub(FEN_LABEL_WIDTH))]
        };
        canvas.put(x, y + BOARD_ROWS + FEN_GAP, FEN_LABEL, Style::new(STEEL));
        let area = widgets::copyable(
            canvas,
            x + FEN_LABEL_WIDTH,
            y + BOARD_ROWS + FEN_GAP,
            &lines,
            DIM,
            frame.state,
        );
        regions.push(Region {
            area,
            target: Target::Copy {
                label: "FEN",
                text: position.fen.clone(),
            },
        });

        y += board_rows(fen_rows);
    }

    let rows = settings.min(area.bottom().saturating_sub(y + 1));

    if rows > 0 {
        widgets::heading(canvas, x, y, width, "Settings");
        widgets::settings(
            canvas,
            Rect::new(x, y + 1, width, rows),
            frame.catalog,
            frame.state,
            regions,
        );
    }

    board_area
}

fn version(engine_name: &str) -> &str {
    engine_name
        .strip_prefix("Valkyrie")
        .map(|rest| rest.trim_start_matches(['-', ' ']))
        .filter(|rest| !rest.is_empty())
        .unwrap_or(engine_name)
}
