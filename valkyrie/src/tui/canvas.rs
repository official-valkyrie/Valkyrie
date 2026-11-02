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
    palette::{BACKGROUND, RULE},
    theme::{PLAIN, Style},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub symbol: char,
    pub style: Style,
}

const BLANK: Cell = Cell {
    symbol: ' ',
    style: PLAIN,
};

const UNSEEN: Cell = Cell {
    symbol: '\0',
    style: PLAIN,
};

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rect {
    pub const fn new(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub const fn bottom(&self) -> usize {
        self.y + self.height
    }

    pub const fn contains(&self, (x, y): (usize, usize)) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.bottom()
    }

    pub const fn shrink(&self, margin: usize) -> Self {
        Self {
            x: self.x + margin,
            y: self.y + margin,
            width: self.width.saturating_sub(margin * 2),
            height: self.height.saturating_sub(margin * 2),
        }
    }
}

#[derive(Debug)]
pub struct Canvas {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![BLANK; width * height],
        }
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub fn resize(&mut self, width: usize, height: usize) -> bool {
        if self.width == width && self.height == height {
            return false;
        }

        *self = Self::new(width, height);
        true
    }

    pub fn clear(&mut self) {
        self.cells.fill(BLANK);
    }

    pub fn set(&mut self, x: usize, y: usize, symbol: char, style: Style) {
        if x < self.width && y < self.height {
            self.cells[y * self.width + x] = Cell { symbol, style };
        }
    }

    pub fn put(&mut self, x: usize, y: usize, text: &str, style: Style) -> usize {
        let mut column = x;

        for symbol in text.chars() {
            self.set(column, y, symbol, style);
            column += 1;
        }

        column
    }

    pub fn fill(&mut self, area: Rect, style: Style) {
        for y in area.y..area.bottom() {
            for x in area.x..area.x + area.width {
                self.set(x, y, ' ', style);
            }
        }
    }

    pub fn frame(&mut self, area: Rect, title: &str, border: Style, title_style: Style) -> Rect {
        if area.width < 2 || area.height < 2 {
            return area.shrink(1);
        }

        let right = area.x + area.width - 1;
        let bottom = area.y + area.height - 1;

        for x in area.x + 1..right {
            self.set(x, area.y, '─', border);
            self.set(x, bottom, '─', border);
        }

        for y in area.y + 1..bottom {
            self.set(area.x, y, '│', border);
            self.set(right, y, '│', border);
        }

        self.set(area.x, area.y, '╭', border);
        self.set(right, area.y, '╮', border);
        self.set(area.x, bottom, '╰', border);
        self.set(right, bottom, '╯', border);

        if !title.is_empty() && area.width > title.chars().count() + 4 {
            self.put(area.x + 2, area.y, &format!(" {title} "), title_style);
        }

        area.shrink(1)
    }

    pub fn fade(&mut self, x: usize, y: usize, amount: f32) {
        if x < self.width && y < self.height {
            let style = &mut self.cells[y * self.width + x].style;

            style.foreground = BACKGROUND.lerp(style.foreground, amount);
            style.background = BACKGROUND.lerp(style.background, amount);
        }
    }

    pub fn dim(&mut self, amount: f32) {
        for cell in &mut self.cells {
            cell.style.foreground = cell.style.foreground.lerp(cell.style.background, amount);
            cell.style.bold = false;
        }
    }

    pub fn heading(&mut self, x: usize, y: usize, width: usize, title: &str, style: Style) {
        let end = self.put(x, y, &fit(title, width), style);

        for column in end + 1..x + width {
            self.set(column, y, '─', Style::new(RULE));
        }
    }

    pub fn diff(&self, shown: &mut Vec<Cell>) -> String {
        if shown.len() != self.cells.len() {
            shown.clear();
            shown.resize(self.cells.len(), UNSEEN);
        }

        let mut out = String::new();
        let mut current: Option<Style> = None;

        for (row, (new, old)) in self
            .cells
            .chunks(self.width.max(1))
            .zip(shown.chunks(self.width.max(1)))
            .enumerate()
        {
            let mut column = 0;

            while column < new.len() {
                if new[column] == old[column] {
                    column += 1;
                    continue;
                }

                out.push_str(&format!("\x1b[{};{}H", row + 1, column + 1));

                while column < new.len() && new[column] != old[column] {
                    let cell = new[column];

                    if current != Some(cell.style) {
                        push_style(&mut out, cell.style);
                        current = Some(cell.style);
                    }

                    out.push(cell.symbol);
                    column += 1;
                }
            }
        }

        if !out.is_empty() {
            out.push_str("\x1b[0m");
        }

        shown.copy_from_slice(&self.cells);
        out
    }
}

fn push_style(out: &mut String, style: Style) {
    let foreground = style.foreground;
    let background = style.background;

    out.push_str(&format!(
        "\x1b[{};38;2;{};{};{};48;2;{};{};{}m",
        if style.bold { 1 } else { 22 },
        foreground.0,
        foreground.1,
        foreground.2,
        background.0,
        background.1,
        background.2,
    ));
}

pub fn fit(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }

    if width == 0 {
        return String::new();
    }

    let mut shortened: String = text.chars().take(width - 1).collect();
    shortened.push('…');
    shortened
}
