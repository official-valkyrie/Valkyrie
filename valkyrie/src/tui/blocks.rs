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
    animation::BoardClock,
    canvas::{Canvas, Rect, fit},
    large_board::{
        self, LABEL_WIDTH, LARGE_BOARD_ROWS, LARGE_BOARD_WIDTH, SQUARE_HEIGHT, SQUARE_WIDTH,
    },
    logo::Logo,
    palette::{GOLD, STEEL},
    perft,
    screens::Frame,
    state::{PerftRun, Region, Target},
    theme::{DIM, Style},
    widgets::{
        self, BOARD_LABEL_WIDTH, BOARD_ROWS, BOARD_SQUARE_WIDTH, BOARD_WIDTH, FEN_LABEL,
        FEN_LABEL_WIDTH,
    },
};

const VERSION_GAP: usize = 3;
const TITLE: &str = "V A L K Y R I E";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Block {
    Logo,
    Title,
    Board(BoardSize),
    Details,
    Fen,
    Settings,
    Response,
    Input,
    Footer,
    PerftHeader,
    Splits,
    Summary,
}

impl Block {
    pub const fn animates_itself(self) -> bool {
        matches!(self, Self::Logo | Self::Board(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardSize {
    Small,
    Large,
}

impl BoardSize {
    pub const fn width(self) -> usize {
        match self {
            Self::Small => BOARD_WIDTH,
            Self::Large => LARGE_BOARD_WIDTH,
        }
    }

    pub const fn rows(self) -> usize {
        match self {
            Self::Small => BOARD_ROWS,
            Self::Large => LARGE_BOARD_ROWS,
        }
    }

    pub const fn label_width(self) -> usize {
        match self {
            Self::Small => BOARD_LABEL_WIDTH,
            Self::Large => LABEL_WIDTH,
        }
    }

    pub const fn square(self) -> (usize, usize) {
        match self {
            Self::Small => (BOARD_SQUARE_WIDTH, 1),
            Self::Large => (SQUARE_WIDTH, SQUARE_HEIGHT),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Placement {
    pub block: Block,
    pub area: Rect,
}

impl Placement {
    pub const fn new(block: Block, area: Rect) -> Self {
        Self { block, area }
    }
}

pub fn draw(
    canvas: &mut Canvas,
    placement: Placement,
    frame: &Frame<'_>,
    regions: &mut Vec<Region>,
) {
    let area = placement.area;
    let state = frame.state;

    match placement.block {
        Block::Logo => Logo::get().draw(canvas, area.x, area.y, frame.progress),
        Block::Title => title(canvas, area, frame, regions),
        Block::Board(size) => board(canvas, area, size, frame),
        Block::Details => {
            if let Some(position) = &state.position {
                widgets::position_details(canvas, area.x, area.y, area.width, position);
            }
        }
        Block::Fen => fen(canvas, area, frame, regions),
        Block::Settings => {
            widgets::heading(canvas, area.x, area.y, area.width, "Settings");
            widgets::settings(
                canvas,
                Rect::new(
                    area.x,
                    area.y + 1,
                    area.width,
                    area.height.saturating_sub(1),
                ),
                frame.catalog,
                state,
                regions,
            );
        }
        Block::Response => widgets::response(canvas, area.x, area.y, area.width, state),
        Block::Input => widgets::input(canvas, area, state, frame.suggestions),
        Block::Footer => widgets::footer(
            canvas,
            area,
            frame.catalog,
            state,
            frame.suggestions,
            regions,
        ),
        Block::PerftHeader => {
            if let Some(run) = &state.perft {
                perft::header(canvas, area, run);
            }
        }
        Block::Splits => {
            if let Some(run) = &state.perft {
                perft::splits(canvas, area, run, perft::tones(state).0);
                copy_region(regions, area, run, perft::RESULTS, perft::results_text);
            }
        }
        Block::Summary => {
            if let Some(run) = &state.perft {
                perft::summary(canvas, area, run, perft::tones(state).1);
                copy_region(regions, area, run, perft::SUMMARY, perft::summary_text);
            }
        }
    }
}

fn copy_region(
    regions: &mut Vec<Region>,
    area: Rect,
    run: &PerftRun,
    label: &'static str,
    text: fn(&PerftRun) -> String,
) {
    if run.done() {
        regions.push(Region {
            area,
            target: Target::Copy {
                label,
                text: text(run),
            },
        });
    }
}

fn title(canvas: &mut Canvas, area: Rect, frame: &Frame<'_>, regions: &mut Vec<Region>) {
    let end = canvas.put(area.x, area.y, TITLE, Style::new(GOLD).bold());
    let version = version(&frame.catalog.engine_name);
    let room = (area.x + area.width).saturating_sub(end + VERSION_GAP);

    if room == 0 {
        return;
    }

    copyable(
        canvas,
        (end + VERSION_GAP, area.y),
        &[fit(version, room)],
        ("version", version),
        frame,
        regions,
    );
}

fn copyable(
    canvas: &mut Canvas,
    (x, y): (usize, usize),
    lines: &[String],
    (label, text): (&'static str, &str),
    frame: &Frame<'_>,
    regions: &mut Vec<Region>,
) {
    let area = widgets::copyable(canvas, x, y, lines, DIM, frame.state);

    regions.push(Region {
        area,
        target: Target::Copy {
            label,
            text: text.to_string(),
        },
    });
}

fn board(canvas: &mut Canvas, area: Rect, size: BoardSize, frame: &Frame<'_>) {
    let Some(position) = &frame.state.position else {
        return;
    };

    let clock = board_clock(area, frame, canvas.height());
    let fade = clock.map_or(1.0, BoardClock::board);
    let opacity = |slot: usize| clock.map_or(1.0, |clock| clock.piece(slot));

    match size {
        BoardSize::Small => widgets::board(
            canvas,
            area.x,
            area.y,
            position,
            fade,
            (!frame.images()).then_some(&opacity as &dyn Fn(usize) -> f32),
        ),
        BoardSize::Large => large_board::draw(canvas, area.x, area.y, position, fade),
    }
}

fn fen(canvas: &mut Canvas, area: Rect, frame: &Frame<'_>, regions: &mut Vec<Region>) {
    let Some(position) = &frame.state.position else {
        return;
    };

    let width = area.width.saturating_sub(FEN_LABEL_WIDTH);
    let lines = widgets::fen_lines(position, width);
    let lines = if lines.len() <= area.height {
        lines
    } else {
        vec![fit(&position.fen, width)]
    };

    canvas.put(area.x, area.y, FEN_LABEL, Style::new(STEEL));

    copyable(
        canvas,
        (area.x + FEN_LABEL_WIDTH, area.y),
        &lines,
        ("FEN", &position.fen),
        frame,
        regions,
    );
}

fn version(engine_name: &str) -> &str {
    engine_name
        .strip_prefix("Valkyrie")
        .map(|rest| rest.trim_start_matches(['-', ' ']))
        .filter(|rest| !rest.is_empty())
        .unwrap_or(engine_name)
}

pub fn board_clock(area: Rect, frame: &Frame<'_>, height: usize) -> Option<BoardClock> {
    frame
        .progress
        .map(|progress| BoardClock::new(frame.state.screen, progress, area.y, height))
        .or_else(|| frame.state.placed.and_then(BoardClock::placing))
}
