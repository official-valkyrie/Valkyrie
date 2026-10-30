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
    fs::File,
    io::{BufRead, BufReader},
};

use valkyrie_chess::{ChessBoard, FEN, perft};

#[test]
fn standard() {
    let file = File::open("./tests/standard.epd").unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let line = line.split(';').collect::<Vec<&str>>();
        let fen = FEN::from(line[0]);
        let target = line[line.len() - 2]
            .split_whitespace()
            .collect::<Vec<&str>>();
        let expected_result = target[1].parse::<u64>().unwrap();
        let depth = target[0].chars().collect::<Vec<char>>()[1] as u8 - b'0';
        println!("{fen}");
        let (result, _) = perft::<true>(&ChessBoard::from(&fen), Some(depth), |_, _| {});
        assert_eq!(result, expected_result);
    }
}

#[test]
fn frc() {
    let file = File::open("./tests/fischer.epd").unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        let line = line.split(';').collect::<Vec<&str>>();
        let fen = FEN::from(line[0]);
        let target = line[line.len() - 3]
            .split_whitespace()
            .collect::<Vec<&str>>();
        let expected_result = target[1].parse::<u64>().unwrap();
        let depth = target[0].chars().collect::<Vec<char>>()[1] as u8 - b'0';
        println!("{fen}");
        let (result, _) = perft::<true>(&ChessBoard::from(&fen), Some(depth), |_, _| {});
        assert_eq!(result, expected_result);
    }
}

#[test]
fn move_counters_match_generator() {
    let mut positions_without_moves = 0;

    for path in ["./tests/standard.epd", "./tests/fischer.epd"] {
        let reader = BufReader::new(File::open(path).unwrap());

        for line in reader.lines() {
            let line = line.unwrap();
            let fen = FEN::from(line.split(';').next().unwrap());
            positions_without_moves += check_move_counters(&ChessBoard::from(&fen), 2);
        }
    }

    assert!(positions_without_moves > 0);
}

fn check_move_counters(board: &ChessBoard, depth: u8) -> u32 {
    let mut moves = Vec::new();
    board.map_legal_moves(|mv| moves.push(mv));

    assert_eq!(board.count_legal_moves() as usize, moves.len(), "{board}");
    assert_eq!(board.has_legal_moves(), !moves.is_empty(), "{board}");

    if depth == 0 || moves.is_empty() {
        return u32::from(moves.is_empty());
    }

    moves
        .into_iter()
        .map(|mv| {
            let mut child = *board;
            child.make_move(mv);
            check_move_counters(&child, depth - 1)
        })
        .sum()
}
