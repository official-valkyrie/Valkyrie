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
    blocks::{self, Block, BoardSize, Placement},
    canvas::{Canvas, Rect},
    completion::{Catalog, Suggestions},
    large_board::{self, SquareImage},
    logo::Logo,
    palette::{GOLD_LIGHT, Rgb},
    perft,
    state::{PerftRun, Region, Screen, State},
    theme::ACCENT,
    widgets::{self, DETAIL_ROWS, FEN_LABEL_WIDTH, INPUT_HEIGHT},
};

const MIN_WIDTH: usize = 44;
const MIN_HEIGHT: usize = 12;
const MARGIN: usize = 2;
const FOOTER_INSET: usize = 1;
const LOGO_GAP: usize = 5;
const PANEL_MIN_WIDTH: usize = 38;
const PANEL_MAX_WIDTH: usize = 46;
const SOLO_PANEL_WIDTH: usize = 60;
const SECTION_GAP: usize = 1;
const TITLE_GAP: usize = 2;
const FEN_GAP: usize = 1;
const RESPONSE_ROWS: usize = 1;
const BOARD_GAP: usize = 4;
const DETAILS_MIN_WIDTH: usize = 18;
const PERFT_TOP: usize = 1;
const PERFT_MAX_WIDTH: usize = 116;
const PERFT_COLUMN_GAP: usize = 5;
const SPLITS_MIN_WIDTH: usize = 42;
const PERFT_ARROW_COLOR: Rgb = GOLD_LIGHT;
const PERFT_ARROW_WIDTH: f32 = 0.16;
const PERFT_ARROW_OPACITY: f32 = 0.8;

pub struct Frame<'a> {
    pub state: &'a State,
    pub catalog: &'a Catalog,
    pub suggestions: &'a Suggestions,
    pub progress: Option<f32>,
}

impl Frame<'_> {
    pub fn images(&self) -> bool {
        self.state.graphics.is_some() && !self.state.help_open
    }

    fn fen_rows(&self, width: usize) -> usize {
        self.state.position.as_ref().map_or(0, |position| {
            widgets::fen_lines(position, width.saturating_sub(FEN_LABEL_WIDTH)).len()
        })
    }
}

pub fn draw(canvas: &mut Canvas, frame: &Frame<'_>) -> (Vec<Region>, Vec<SquareImage>) {
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
        return (regions, Vec::new());
    }

    let body = Rect::new(MARGIN, 0, width - MARGIN * 2, height - 1);
    let mut placements = match frame.state.screen {
        Screen::Welcome => welcome(body, frame),
        Screen::Perft => perft(body, frame),
    };

    placements.push(Placement::new(
        Block::Footer,
        Rect::new(
            body.x + FOOTER_INSET,
            body.bottom(),
            body.width - FOOTER_INSET * 2,
            1,
        ),
    ));

    for placement in &placements {
        blocks::draw(canvas, *placement, frame, &mut regions);
    }

    if let Some(input) = placements
        .iter()
        .find(|placement| placement.block == Block::Input)
    {
        widgets::suggestions(
            canvas,
            input.area,
            frame.state,
            frame.suggestions,
            &mut regions,
        );
    }

    if frame.state.help_open {
        widgets::help(canvas, frame.catalog);
    }

    if let Some(progress) = frame.progress {
        let animated: Vec<Rect> = placements
            .iter()
            .filter(|placement| placement.block.animates_itself())
            .map(|placement| placement.area)
            .collect();

        animation::reveal_rows(canvas, progress, &animated);
    }

    let pieces = placements
        .iter()
        .find_map(|placement| match placement.block {
            Block::Board(size) if frame.images() => {
                Some(board_images(placement.area, size, frame, height))
            }
            _ => None,
        })
        .unwrap_or_default();

    (regions, pieces)
}

fn board_images(area: Rect, size: BoardSize, frame: &Frame<'_>, height: usize) -> Vec<SquareImage> {
    let Some(position) = &frame.state.position else {
        return Vec::new();
    };

    let clock = blocks::board_clock(area, frame, height);
    let opacity = |slot: usize| {
        let fade = clock.map_or(1.0, |clock| clock.piece(slot));
        (fade * f32::from(animation::PIECE_STEPS)).round() as u8
    };
    let arrow = clock
        .is_none()
        .then(|| frame.state.perft.as_ref().and_then(PerftRun::current))
        .flatten()
        .map(|arrow| {
            arrow
                .color(PERFT_ARROW_COLOR)
                .width(PERFT_ARROW_WIDTH)
                .opacity(PERFT_ARROW_OPACITY)
        });

    large_board::images(
        area,
        size.label_width(),
        size.square(),
        position,
        opacity,
        arrow,
    )
}

fn perft(body: Rect, frame: &Frame<'_>) -> Vec<Placement> {
    let span = body.width.min(PERFT_MAX_WIDTH);
    let x = body.x + (body.width - span) / 2;
    let header = body.y + PERFT_TOP;
    let main = header + 1 + TITLE_GAP;
    let large = frame.state.graphics.is_some()
        && main + BoardSize::Large.rows() <= body.bottom()
        && span >= BoardSize::Large.width() + PERFT_COLUMN_GAP + SPLITS_MIN_WIDTH;
    let size = if large {
        BoardSize::Large
    } else {
        BoardSize::Small
    };

    let right = x + size.width() + PERFT_COLUMN_GAP;
    let right_width = span - size.width() - PERFT_COLUMN_GAP;
    let summary = body
        .bottom()
        .saturating_sub(SECTION_GAP + perft::SUMMARY_ROWS);
    let splits = summary.saturating_sub(SECTION_GAP + main).max(2);

    let mut placements = vec![
        Placement::new(Block::PerftHeader, Rect::new(x, header, span, 1)),
        Placement::new(
            Block::Board(size),
            Rect::new(x, main, size.width(), size.rows()),
        ),
        Placement::new(Block::Splits, Rect::new(right, main, right_width, splits)),
        Placement::new(
            Block::Summary,
            Rect::new(right, summary, right_width, perft::SUMMARY_ROWS),
        ),
    ];

    let fen_y = main + size.rows() + FEN_GAP;
    let fen = frame.fen_rows(size.width());

    if fen > 0 && fen_y + fen < body.bottom() {
        placements.push(Placement::new(
            Block::Fen,
            Rect::new(x, fen_y, size.width(), fen),
        ));
    }

    placements
}

fn welcome(body: Rect, frame: &Frame<'_>) -> Vec<Placement> {
    let content_height = body.height - INPUT_HEIGHT - RESPONSE_ROWS;
    let logo = Logo::get();
    let panel_width = body
        .width
        .saturating_sub(logo.width + LOGO_GAP)
        .min(PANEL_MAX_WIDTH);
    let mut placements = Vec::new();

    if panel_width >= PANEL_MIN_WIDTH && content_height >= logo.height {
        let x = body.x + (body.width - logo.width - LOGO_GAP - panel_width) / 2;
        let y = body.y + (content_height - logo.height) / 2;

        placements.push(Placement::new(
            Block::Logo,
            Rect::new(x, y, logo.width, logo.height),
        ));
        panel(
            &mut placements,
            frame,
            Rect::new(x + logo.width + LOGO_GAP, y, panel_width, logo.height),
        );
    } else {
        let panel_width = body.width.min(SOLO_PANEL_WIDTH);

        panel(
            &mut placements,
            frame,
            Rect::new(
                body.x + (body.width - panel_width) / 2,
                body.y,
                panel_width,
                content_height,
            ),
        );
    }

    let input = Rect::new(
        body.x,
        body.y + content_height + RESPONSE_ROWS,
        body.width,
        INPUT_HEIGHT,
    );

    placements.extend([
        Placement::new(
            Block::Response,
            Rect::new(
                input.x + FOOTER_INSET,
                body.y + content_height,
                input.width - FOOTER_INSET * 2,
                RESPONSE_ROWS,
            ),
        ),
        Placement::new(Block::Input, input),
    ]);

    placements
}

fn panel(placements: &mut Vec<Placement>, frame: &Frame<'_>, area: Rect) {
    let (x, width) = (area.x, area.width);
    let size = BoardSize::Small;
    let fen_rows = frame.fen_rows(width);
    let settings = widgets::visible_options(frame.catalog).count();

    let title_rows = 1 + TITLE_GAP;
    let board_rows = |fen_rows: usize| size.rows() + FEN_GAP + fen_rows + SECTION_GAP;
    let total = |board: Option<usize>| title_rows + board.map_or(0, board_rows) + 1 + settings;

    let board = frame.state.position.as_ref().map(|_| fen_rows);
    let board = [board, board.map(|_| 1), None]
        .into_iter()
        .find(|board| total(*board) <= area.height)
        .flatten();

    let mut y = area.y + area.height.saturating_sub(total(board)) / 2;

    placements.push(Placement::new(Block::Title, Rect::new(x, y, width, 1)));
    y += title_rows;

    if let Some(fen_rows) = board {
        placements.push(Placement::new(
            Block::Board(size),
            Rect::new(x, y, size.width(), size.rows()),
        ));

        if width >= size.width() + BOARD_GAP + DETAILS_MIN_WIDTH {
            placements.push(Placement::new(
                Block::Details,
                Rect::new(
                    x + size.width() + BOARD_GAP,
                    y,
                    width - size.width() - BOARD_GAP,
                    DETAIL_ROWS,
                ),
            ));
        }

        placements.push(Placement::new(
            Block::Fen,
            Rect::new(x, y + size.rows() + FEN_GAP, width, fen_rows),
        ));

        y += board_rows(fen_rows);
    }

    let rows = settings.min(area.bottom().saturating_sub(y + 1));

    if rows > 0 {
        placements.push(Placement::new(
            Block::Settings,
            Rect::new(x, y, width, rows + 1),
        ));
    }
}
