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

use crate::{
    GameState, Move,
    board::{chess_board::ChessBoard, move_history::MoveHistory},
};

#[derive(Debug, Clone, Copy)]
pub struct ChessPosition {
    board: ChessBoard,
    history: MoveHistory,
    castle_mask: [u8; 64],
}

impl ChessPosition {
    #[inline]
    pub fn board(&self) -> &ChessBoard {
        &self.board
    }

    #[inline]
    pub fn history(&self) -> &MoveHistory {
        &self.history
    }

    #[inline]
    pub fn reset_history(&mut self) {
        self.history.reset()
    }

    #[inline]
    pub fn make_move(&mut self, mv: Move) {
        self.board.make_move_with_mask(mv, &self.castle_mask);

        if self.board.half_moves() == 0 {
            self.history.reset()
        }

        self.history.push(self.board.hash())
    }

    pub fn game_state(&self, root: &ChessPosition) -> GameState {
        if !self.board.has_legal_moves() {
            return if self.board.is_in_check() {
                GameState::Lost(0)
            } else {
                GameState::Drew
            };
        }

        if self.board.is_fifty_move_rule() || self.board.is_insufficient_material() {
            return GameState::Drew;
        }

        let key = self.board.hash();
        let root_repetitions = root.history.get_repetitions(key);
        let search_repetitions = self.history.get_repetitions(key) - root_repetitions;

        if root_repetitions >= 3
            || search_repetitions >= 2
            || root_repetitions + search_repetitions >= 3
        {
            GameState::Drew
        } else {
            GameState::Ongoing
        }
    }

    #[inline]
    pub fn is_draw(&self, root: &ChessPosition) -> bool {
        self.game_state(root) == GameState::Drew
    }
}

impl From<ChessBoard> for ChessPosition {
    fn from(value: ChessBoard) -> Self {
        let mut position = Self {
            board: value,
            history: MoveHistory::default(),
            castle_mask: value.castle_rights().get_castle_mask(),
        };

        position.history.push(value.hash());

        position
    }
}
