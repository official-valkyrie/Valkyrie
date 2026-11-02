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

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::tui::completion::{Catalog, suggest};

const HISTORY_CAPACITY: usize = 200;

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Nothing,
    Submit(String),
    Interrupt,
}

#[derive(Debug, Default)]
pub struct Editor {
    buffer: Vec<char>,
    cursor: usize,
    history: Vec<String>,
    recall: Option<usize>,
    draft: String,
    selected: usize,
    chosen: bool,
}

impl Editor {
    pub fn text(&self) -> String {
        self.buffer.iter().collect()
    }

    pub fn before_cursor(&self) -> String {
        self.buffer[..self.cursor].iter().collect()
    }

    pub const fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn at_end(&self) -> bool {
        self.cursor == self.buffer.len()
    }

    pub const fn selected(&self) -> usize {
        self.selected
    }

    pub fn insert(&mut self, text: &str) {
        for symbol in text.chars() {
            let symbol = if symbol.is_control() { ' ' } else { symbol };

            self.buffer.insert(self.cursor, symbol);
            self.cursor += 1;
        }

        self.edited();
    }

    pub fn handle(&mut self, key: KeyEvent, catalog: &Catalog) -> Outcome {
        let control = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let word = control || alt;

        match key.code {
            KeyCode::Char(symbol) if control => match symbol.to_ascii_lowercase() {
                'c' => return Outcome::Interrupt,
                'a' => self.cursor = 0,
                'e' => self.cursor = self.buffer.len(),
                'b' => self.cursor = self.cursor.saturating_sub(1),
                'f' => self.cursor = (self.cursor + 1).min(self.buffer.len()),
                'u' => self.delete_range(0, self.cursor),
                'k' => self.delete_range(self.cursor, self.buffer.len()),
                'w' | 'h' => self.delete_word_back(),
                'p' => self.move_selection(catalog, false),
                'n' => self.move_selection(catalog, true),
                _ => {}
            },
            KeyCode::Char(symbol) if alt => match symbol {
                'b' => self.cursor = self.word_start(),
                'f' => self.cursor = self.word_end(),
                _ => {}
            },
            KeyCode::Char(symbol) => self.insert(&symbol.to_string()),
            KeyCode::Backspace if word => self.delete_word_back(),
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.delete_range(self.cursor - 1, self.cursor);
                }
            }
            KeyCode::Delete => {
                if self.cursor < self.buffer.len() {
                    self.delete_range(self.cursor, self.cursor + 1);
                }
            }
            KeyCode::Left if word => self.cursor = self.word_start(),
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right if word => self.cursor = self.word_end(),
            KeyCode::Right => {
                if self.at_end() {
                    self.accept_ghost(catalog);
                } else {
                    self.cursor += 1;
                }
            }
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.buffer.len(),
            KeyCode::Up => self.move_selection(catalog, false),
            KeyCode::Down => self.move_selection(catalog, true),
            KeyCode::Tab => {
                self.accept(catalog);
            }
            KeyCode::BackTab => self.move_selection(catalog, false),
            KeyCode::Esc => {
                self.buffer.clear();
                self.cursor = 0;
                self.edited();
            }
            KeyCode::Enter => return self.submit(catalog),
            _ => {}
        }

        Outcome::Nothing
    }

    fn edited(&mut self) {
        self.recall = None;
        self.selected = 0;
        self.chosen = false;
    }

    fn delete_range(&mut self, from: usize, to: usize) {
        self.buffer.drain(from..to);
        self.cursor = from;
        self.edited();
    }

    fn word_start(&self) -> usize {
        let mut position = self.cursor;

        while position > 0 && self.buffer[position - 1].is_whitespace() {
            position -= 1;
        }

        while position > 0 && !self.buffer[position - 1].is_whitespace() {
            position -= 1;
        }

        position
    }

    fn word_end(&self) -> usize {
        let mut position = self.cursor;

        while position < self.buffer.len() && self.buffer[position].is_whitespace() {
            position += 1;
        }

        while position < self.buffer.len() && !self.buffer[position].is_whitespace() {
            position += 1;
        }

        position
    }

    fn delete_word_back(&mut self) {
        let start = self.word_start();

        self.delete_range(start, self.cursor);
    }

    fn move_selection(&mut self, catalog: &Catalog, forward: bool) {
        let count = suggest(catalog, &self.before_cursor()).items.len();

        if count > 0 && !self.buffer.is_empty() && self.recall.is_none() {
            self.selected = if forward {
                (self.selected + 1) % count
            } else {
                (self.selected + count - 1) % count
            };
            self.chosen = true;
            return;
        }

        self.navigate_history(forward);
    }

    fn navigate_history(&mut self, forward: bool) {
        if self.history.is_empty() {
            return;
        }

        let index = match (self.recall, forward) {
            (None, true) => return,
            (None, false) => {
                self.draft = self.text();
                self.history.len() - 1
            }
            (Some(index), false) => index.saturating_sub(1),
            (Some(index), true) if index + 1 < self.history.len() => index + 1,
            (Some(_), true) => {
                self.buffer = self.draft.chars().collect();
                self.cursor = self.buffer.len();
                self.recall = None;
                return;
            }
        };

        self.buffer = self.history[index].chars().collect();
        self.cursor = self.buffer.len();
        self.recall = Some(index);
        self.selected = 0;
        self.chosen = false;
    }

    pub fn choose(&mut self, index: usize, catalog: &Catalog) {
        self.selected = index;
        self.accept(catalog);
    }

    fn accept(&mut self, catalog: &Catalog) -> bool {
        let suggestions = suggest(catalog, &self.before_cursor());

        let Some(item) = suggestions
            .items
            .get(self.selected.min(suggestions.items.len().saturating_sub(1)))
        else {
            return false;
        };

        let insertion: Vec<char> = item.insert.chars().collect();
        let length = insertion.len();

        self.buffer
            .splice(suggestions.start..self.cursor, insertion);
        self.cursor = suggestions.start + length;
        self.edited();
        true
    }

    fn accept_ghost(&mut self, catalog: &Catalog) {
        let suggestions = suggest(catalog, &self.before_cursor());
        let typed: String = self.buffer[suggestions.start.min(self.cursor)..]
            .iter()
            .collect();

        if suggestions.ghost(&typed, self.selected).is_some() {
            self.accept(catalog);
        }
    }

    fn submit(&mut self, catalog: &Catalog) -> Outcome {
        if self.chosen && self.accept(catalog) {
            return Outcome::Nothing;
        }

        let line = self.text().trim().to_string();

        self.buffer.clear();
        self.cursor = 0;
        self.edited();

        if line.is_empty() {
            return Outcome::Nothing;
        }

        if self.history.last() != Some(&line) {
            self.history.push(line.clone());

            if self.history.len() > HISTORY_CAPACITY {
                self.history.remove(0);
            }
        }

        Outcome::Submit(line)
    }
}
