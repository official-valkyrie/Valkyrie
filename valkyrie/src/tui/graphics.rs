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

use std::{collections::HashMap, env, fmt::Write};

use crate::{
    terminal,
    tui::{
        animation::PIECE_STEPS,
        large_board::{PieceImage, SquareImage},
        palette::{GOLD, PIECE_OUTLINE, Rgb, WHITE},
    },
};

const PIECE_SIZE: usize = 128;
const PIECE_SCALE: f32 = 0.92;
const DETAIL_SIZE: f32 = 48.0;
const MIN_GAMMA: f32 = 0.45;
const ALPHA_CUTOFF: f32 = 0.08;
const KITTY_SQUARE: usize = 64;
const KITTY_CHUNK: usize = 4096;
const KITTY_ID_STRIDE: usize = 1000;
const VIRTUAL_CELL: (usize, usize) = (10, 20);
const SIXEL_BAND: usize = 6;
const SIXEL_OFFSET: u8 = 63;
const PALETTE_LIMIT: usize = 256;
const PIECES: [&[u8]; 6] = [
    include_bytes!("assets/pawn.la"),
    include_bytes!("assets/knight.la"),
    include_bytes!("assets/bishop.la"),
    include_bytes!("assets/rook.la"),
    include_bytes!("assets/queen.la"),
    include_bytes!("assets/king.la"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Kitty,
    Sixel { cell: (usize, usize) },
}

impl Protocol {
    pub fn detect() -> Option<Self> {
        let read = |name: &str| env::var(name).unwrap_or_default().to_lowercase();
        let (term, program) = (read("TERM"), read("TERM_PROGRAM"));

        if env::var_os("KITTY_WINDOW_ID").is_some()
            || term.contains("kitty")
            || term.contains("ghostty")
            || matches!(program.as_str(), "wezterm" | "ghostty")
        {
            return Some(Self::Kitty);
        }

        let sixel = env::var_os("WT_SESSION").is_some()
            || (terminal::pseudo_console() && program != "vscode")
            || env::var_os("KONSOLE_VERSION").is_some()
            || term.contains("foot")
            || term.contains("mlterm");

        sixel.then(|| Self::Sixel {
            cell: cell_pixels().unwrap_or(VIRTUAL_CELL),
        })
    }

    pub fn encode(self, image: &SquareImage) -> String {
        let place = format!("\x1b[{};{}H", image.y + 1, image.x + 1);

        match self {
            Self::Kitty => format!("{place}{}", kitty(image)),
            Self::Sixel { cell } => format!(
                "{place}{}",
                sixel(
                    image,
                    image.columns * cell.0,
                    image.rows * cell.1 / SIXEL_BAND * SIXEL_BAND,
                )
            ),
        }
    }

    pub fn erase(self, image: &SquareImage) -> String {
        match self {
            Self::Kitty => format!("\x1b_Ga=d,d=i,i={},q=2\x1b\\", kitty_id(image)),
            Self::Sixel { .. } => self.encode(&image.plain()),
        }
    }

    pub fn clear(self) -> String {
        match self {
            Self::Kitty => "\x1b_Ga=d,d=A,q=2\x1b\\".to_string(),
            Self::Sixel { .. } => String::new(),
        }
    }
}

fn cell_pixels() -> Option<(usize, usize)> {
    if env::var_os("WT_SESSION").is_some() {
        return None;
    }

    let size = crossterm::terminal::window_size().ok()?;

    (size.width > 0 && size.height > 0 && size.columns > 0 && size.rows > 0).then(|| {
        (
            usize::from(size.width / size.columns),
            usize::from(size.height / size.rows),
        )
    })
}

fn pixels(image: &SquareImage, width: usize, height: usize) -> Vec<(Rgb, f32)> {
    let mut pixels = vec![(image.square, 0.0); width * height];

    if let Some(piece) = image.piece {
        paint_piece(&mut pixels, piece, width, height);
    }

    if let Some(arrow) = image.arrow {
        let unit = width as f32;
        let offset = (width as f32 - height as f32) / 2.0;
        let (row, file) = image.shown;

        for y in 0..height {
            for x in 0..width {
                let point = (
                    file as f32 + (x as f32 + 0.5) / unit,
                    row as f32 + (y as f32 + 0.5 + offset) / unit,
                );
                let alpha = arrow.alpha(point, unit);

                if alpha > 0.0 {
                    let pixel = &mut pixels[y * width + x];
                    *pixel = over((arrow.color, alpha), *pixel);
                }
            }
        }
    }

    pixels
}

fn paint_piece(pixels: &mut [(Rgb, f32)], piece: PieceImage, width: usize, height: usize) {
    let size = (width.min(height) as f32 * PIECE_SCALE) as usize;
    let origin = ((width - size) / 2, (height - size) / 2);
    let fill = if piece.white { WHITE } else { GOLD };
    let opacity = f32::from(piece.opacity) / f32::from(PIECE_STEPS);
    let gamma = (size as f32 / DETAIL_SIZE).clamp(MIN_GAMMA, 1.0);

    for y in 0..size {
        for x in 0..size {
            let (alpha, light) = sample(PIECES[piece.piece], x, y, size);

            if alpha >= ALPHA_CUTOFF {
                pixels[(origin.1 + y) * width + origin.0 + x] =
                    (PIECE_OUTLINE.lerp(fill, light.powf(gamma)), alpha * opacity);
            }
        }
    }
}

fn over((top, alpha): (Rgb, f32), (bottom, below): (Rgb, f32)) -> (Rgb, f32) {
    let combined = alpha + below * (1.0 - alpha);

    if combined <= 0.0 {
        return (top, 0.0);
    }

    (bottom.lerp(top, alpha / combined), combined)
}

fn sample(piece: &[u8], x: usize, y: usize, size: usize) -> (f32, f32) {
    let from = |value: usize| value * PIECE_SIZE / size;
    let to = |value: usize| ((value + 1) * PIECE_SIZE / size).max(from(value) + 1);
    let (mut alpha, mut light, mut count) = (0.0, 0.0, 0.0);

    for source_y in from(y)..to(y).min(PIECE_SIZE) {
        for source_x in from(x)..to(x).min(PIECE_SIZE) {
            let index = (source_y * PIECE_SIZE + source_x) * 2;
            let coverage = f32::from(piece[index]) / 255.0;

            alpha += coverage;
            light += coverage * f32::from(piece[index + 1]) / 255.0;
            count += 1.0;
        }
    }

    if alpha <= 0.0 {
        return (0.0, 0.0);
    }

    (alpha / count, light / alpha)
}

fn kitty_id(image: &SquareImage) -> usize {
    1 + image.y * KITTY_ID_STRIDE + image.x
}

fn kitty(image: &SquareImage) -> String {
    let bytes: Vec<u8> = pixels(image, KITTY_SQUARE, KITTY_SQUARE)
        .iter()
        .flat_map(|(color, alpha)| [color.0, color.1, color.2, (alpha * 255.0) as u8])
        .collect();
    let encoded = terminal::base64(&bytes);
    let chunks: Vec<&str> = encoded
        .as_bytes()
        .chunks(KITTY_CHUNK)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or_default())
        .collect();
    let id = kitty_id(image);
    let mut out = String::new();

    for (index, chunk) in chunks.iter().enumerate() {
        let more = u8::from(index + 1 < chunks.len());

        if index == 0 {
            let _ = write!(
                out,
                "\x1b_Ga=T,f=32,s={KITTY_SQUARE},v={KITTY_SQUARE},c={},r={},i={id},p=1,C=1,q=2,m={more};{chunk}\x1b\\",
                image.columns, image.rows
            );
        } else {
            let _ = write!(out, "\x1b_Gm={more};{chunk}\x1b\\");
        }
    }

    out
}

fn sixel(image: &SquareImage, width: usize, height: usize) -> String {
    let colors: Vec<Option<Rgb>> = pixels(image, width, height)
        .into_iter()
        .map(|(color, alpha)| Some(image.square.lerp(color, alpha)))
        .collect();
    let (palette, indices) = quantize(&colors);
    let mut out = format!("\x1bP0;1q\"1;1;{width};{height}");

    for (index, color) in palette.iter().enumerate() {
        let percent = |channel: u8| (u32::from(channel) * 100 + 127) / 255;
        let _ = write!(
            out,
            "#{index};2;{};{};{}",
            percent(color.0),
            percent(color.1),
            percent(color.2)
        );
    }

    for band in (0..height).step_by(SIXEL_BAND) {
        let rows = (height - band).min(SIXEL_BAND);
        let mut used: Vec<usize> = (band..band + rows)
            .flat_map(|y| {
                indices[y * width..(y + 1) * width]
                    .iter()
                    .flatten()
                    .copied()
            })
            .collect();
        used.sort_unstable();
        used.dedup();

        for color in &used {
            let column = |x: usize| {
                (0..rows).fold(0u8, |bits, row| {
                    bits | (u8::from(indices[(band + row) * width + x] == Some(*color)) << row)
                })
            };

            let _ = write!(out, "#{color}");
            let mut x = 0;

            while x < width {
                let bits = column(x);
                let mut run = 1;

                while x + run < width && column(x + run) == bits {
                    run += 1;
                }

                let symbol = char::from(SIXEL_OFFSET + bits);

                if run > 3 {
                    let _ = write!(out, "!{run}{symbol}");
                } else {
                    (0..run).for_each(|_| out.push(symbol));
                }

                x += run;
            }

            out.push('$');
        }

        out.push('-');
    }

    out.push_str("\x1b\\");
    out
}

fn quantize(pixels: &[Option<Rgb>]) -> (Vec<Rgb>, Vec<Option<usize>>) {
    for shift in 0..8 {
        let key = |color: &Rgb| (color.0 >> shift, color.1 >> shift, color.2 >> shift);
        let mut lookup: HashMap<(u8, u8, u8), usize> = HashMap::new();
        let mut palette = Vec::new();

        for color in pixels.iter().flatten() {
            lookup.entry(key(color)).or_insert_with(|| {
                palette.push(*color);
                palette.len() - 1
            });
        }

        if palette.len() <= PALETTE_LIMIT {
            let indices = pixels
                .iter()
                .map(|pixel| pixel.as_ref().map(|color| lookup[&key(color)]))
                .collect();
            return (palette, indices);
        }
    }

    (Vec::new(), vec![None; pixels.len()])
}
