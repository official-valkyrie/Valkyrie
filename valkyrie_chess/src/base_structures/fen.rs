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

use std::{
    char,
    fmt::{Display, Formatter, Result},
};

use crate::{Side, base_structures::CastleRights};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FEN {
    pub(crate) board: [String; 8],
    pub(crate) side_to_move: String,
    pub castle_rights: String,
    pub(crate) en_passant_square: String,
    pub(crate) half_move_counter: String,
    pub(crate) full_move_counter: String,
}

impl FEN {
    pub(crate) fn home_rank(&self, side: Side) -> [char; 8] {
        let rank = &self.board[if side == Side::WHITE { 7 } else { 0 }];
        let mut result = ['.'; 8];
        let mut file = 0;

        for symbol in rank.chars() {
            if let Some(empty) = symbol.to_digit(10) {
                file += empty as usize;
            } else {
                if file < 8 {
                    result[file] = symbol;
                }
                file += 1;
            }
        }

        result
    }

    pub fn start_position() -> Self {
        Self::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1")
    }

    pub fn kiwipete_position() -> Self {
        Self::from("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w HAha - 0 1")
    }
}

impl From<String> for FEN {
    fn from(value: String) -> Self {
        FEN::from(value.as_str())
    }
}

impl From<&str> for FEN {
    fn from(value: &str) -> Self {
        let mut result: Self = Self::default();
        let fen_parts: Vec<&str> = value.split_whitespace().collect();

        let board_parts: Vec<&str> = fen_parts[0].split('/').collect();
        for (index, part) in board_parts.into_iter().enumerate() {
            result.board[index] = part.to_string()
        }

        result.side_to_move = fen_parts[1].to_string();
        result.castle_rights = normalize_castle_rights(&result, fen_parts[2]);

        result.en_passant_square = fen_parts[3].to_string();

        result.half_move_counter = if fen_parts.len() > 4 {
            fen_parts[4]
        } else {
            "0"
        }
        .to_string();

        result.full_move_counter = if fen_parts.len() > 5 {
            fen_parts[5]
        } else {
            "1"
        }
        .to_string();

        result
    }
}

impl From<FEN> for String {
    fn from(value: FEN) -> Self {
        format!(
            "{}/{}/{}/{}/{}/{}/{}/{} {} {} {} {} {}",
            value.board[0],
            value.board[1],
            value.board[2],
            value.board[3],
            value.board[4],
            value.board[5],
            value.board[6],
            value.board[7],
            value.side_to_move,
            value.castle_rights,
            value.en_passant_square,
            value.half_move_counter,
            value.full_move_counter
        )
    }
}

impl Display for FEN {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", String::from(self.clone()))
    }
}

fn normalize_castle_rights(fen: &FEN, rights: &str) -> String {
    let white_rank = fen.home_rank(Side::WHITE);
    let black_rank = fen.home_rank(Side::BLACK);

    let result: String = rights
        .chars()
        .filter_map(|right| {
            let is_white = right.is_ascii_uppercase();
            let home_rank = if is_white { &white_rank } else { &black_rank };
            let (file, _) = CastleRights::resolve(home_rank, right)?;
            Some(char::from(if is_white { b'A' } else { b'a' } + file))
        })
        .collect();

    if result.is_empty() {
        String::from("-")
    } else {
        result
    }
}
