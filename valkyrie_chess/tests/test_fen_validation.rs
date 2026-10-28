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

use std::panic;

use valkyrie_chess::{ChessBoard, FEN, FenViolation, Side};

const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

fn assert_all_accepted(fens: &[&str]) {
    let rejected: Vec<String> = fens
        .iter()
        .map(|fen| (fen, FEN::validate_fen(fen)))
        .filter(|(_, violations)| !violations.is_empty())
        .map(|(fen, violations)| format!("{fen}  ->  {violations:?}"))
        .collect();

    assert!(
        rejected.is_empty(),
        "validator rejected valid FENs:\n{}",
        rejected.join("\n")
    );
}

fn assert_all_report(cases: &[(&str, FenViolation)]) {
    let missed: Vec<String> = cases
        .iter()
        .map(|(fen, expected)| (fen, expected, FEN::validate_fen(fen)))
        .filter(|(_, expected, violations)| !violations.contains(expected))
        .map(|(fen, expected, violations)| {
            format!("{fen}  ->  expected {expected:?}, got {violations:?}")
        })
        .collect();

    assert!(
        missed.is_empty(),
        "validator missed violations:\n{}",
        missed.join("\n")
    );
}

fn expand_placement(placement: &str) -> Option<Vec<char>> {
    let ranks: Vec<&str> = placement.split('/').collect();
    if ranks.len() != 8 {
        return None;
    }

    let mut squares = Vec::with_capacity(64);

    for rank in ranks {
        let mut width = 0;

        for symbol in rank.chars() {
            match symbol {
                '1'..='8' => {
                    let empty = symbol as usize - '0' as usize;
                    squares.extend(std::iter::repeat_n('.', empty));
                    width += empty;
                }
                'p' | 'n' | 'b' | 'r' | 'q' | 'k' | 'P' | 'N' | 'B' | 'R' | 'Q' | 'K' => {
                    squares.push(symbol);
                    width += 1;
                }
                _ => return None,
            }
        }

        if width != 8 {
            return None;
        }
    }

    Some(squares)
}

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

fn fuzzed_fens(count: usize) -> Vec<String> {
    const BASES: [&str; 4] = [
        START,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
        "rrkrrrrr/pp3ppp/3pp3/2p5/5P2/P2P4/1PP1P1PP/RRRRKRRR w FBda - 2 9",
    ];
    const POOL: [char; 40] = [
        'p', 'n', 'b', 'r', 'q', 'k', 'P', 'N', 'B', 'R', 'Q', 'K', '0', '1', '2', '3', '4', '5',
        '6', '7', '8', '9', '/', ' ', '-', 'w', 'a', 'c', 'e', 'h', 'x', 'X', 'z', '.', '+', '#',
        '\t', 'é', '♔', '|',
    ];

    let mut rng = XorShift(0x2545_F491_4F6C_DD1D);

    (0..count)
        .map(|_| {
            let mut fen: Vec<char> = BASES[rng.below(BASES.len())].chars().collect();

            for _ in 0..=rng.below(3) {
                let position = rng.below(fen.len() + 1);

                match rng.below(3) {
                    0 if position < fen.len() => fen[position] = POOL[rng.below(POOL.len())],
                    1 => fen.insert(position, POOL[rng.below(POOL.len())]),
                    _ if position < fen.len() => {
                        fen.remove(position);
                    }
                    _ => {}
                }
            }

            fen.into_iter().collect()
        })
        .collect()
}

#[test]
fn accepts_valid_fens() {
    assert_all_accepted(&[
        START,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3",
        "rrkrrrrr/pp3ppp/3pp3/2p5/5P2/P2P4/1PP1P1PP/RRRRKRRR w FBda - 2 9",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq -",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR   w  KQkq  -  0  1",
        "4k3/8/8/8/8/8/8/4K3 b - - 99 120",
        "8/8/8/8/8/8/8/K6k w - - 0 1",
        "4k3/8/8/8/8/8/8/4K2r w - - 0 1",
        "r3k2r/8/8/8/8/8/8/R3K2R w Kq - 0 1",
    ]);
}

#[test]
fn accepts_all_perft_positions() {
    for path in ["./tests/standard.epd", "./tests/fischer.epd"] {
        let contents = std::fs::read_to_string(path).unwrap();
        let fens: Vec<&str> = contents
            .lines()
            .filter_map(|line| line.split(';').next())
            .map(str::trim)
            .filter(|fen| !fen.is_empty())
            .collect();

        assert_all_accepted(&fens);
    }
}

#[test]
fn reports_missing_fields() {
    assert_all_report(&[
        ("", FenViolation::MissingFields(0)),
        ("   ", FenViolation::MissingFields(0)),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            FenViolation::MissingFields(1),
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w",
            FenViolation::MissingFields(2),
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq",
            FenViolation::MissingFields(3),
        ),
    ]);
}

#[test]
fn reports_extra_fields() {
    assert_all_report(&[
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 extra",
            FenViolation::ExtraFields(7),
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 0",
            FenViolation::ExtraFields(7),
        ),
    ]);
}

#[test]
fn reports_wrong_rank_count() {
    assert_all_report(&[
        (
            "rnbqkbnr/pppppppp/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            FenViolation::WrongRankCount(7),
        ),
        (
            "4k3/8/8/8/8/8/8/8/4K3 w - - 0 1",
            FenViolation::WrongRankCount(9),
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR/ w KQkq - 0 1",
            FenViolation::WrongRankCount(9),
        ),
        (
            "rnbqkbnrpppppppp8888PPPPPPPPRNBQKBNR w - - 0 1",
            FenViolation::WrongRankCount(1),
        ),
    ]);
}

#[test]
fn reports_invalid_ranks() {
    assert_all_report(&[
        (
            "rnbqkbnr/ppppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
            FenViolation::InvalidRank(7),
        ),
        (
            "rnbqkbnr/ppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
            FenViolation::InvalidRank(7),
        ),
        (
            "rnbqkbnr/pppppppp/9/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
            FenViolation::InvalidRank(6),
        ),
        (
            "rnbqkbnr/pppppppp/7/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
            FenViolation::InvalidRank(6),
        ),
        (
            "rnbqkbnr/pppppppp/44p/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
            FenViolation::InvalidRank(6),
        ),
        (
            "rnbqkbnr/pppppppp/0p7/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
            FenViolation::InvalidRank(6),
        ),
        (
            "rnbqkbnr/pppppppp//8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
            FenViolation::InvalidRank(6),
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBXKBNR w - - 0 1",
            FenViolation::InvalidRank(1),
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNz w - - 0 1",
            FenViolation::InvalidRank(1),
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN♔ w - - 0 1",
            FenViolation::InvalidRank(1),
        ),
    ]);
}

#[test]
fn reports_wrong_king_count() {
    assert_all_report(&[
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQQBNR w - - 0 1",
            FenViolation::KingCount(Side::WHITE, 0),
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKKBNR w - - 0 1",
            FenViolation::KingCount(Side::WHITE, 2),
        ),
        (
            "rnbqqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
            FenViolation::KingCount(Side::BLACK, 0),
        ),
        (
            "rnbkkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1",
            FenViolation::KingCount(Side::BLACK, 2),
        ),
        (
            "8/8/8/8/8/8/8/8 w - - 0 1",
            FenViolation::KingCount(Side::WHITE, 0),
        ),
        (
            "8/8/8/8/8/8/8/8 w - - 0 1",
            FenViolation::KingCount(Side::BLACK, 0),
        ),
    ]);
}

#[test]
fn reports_impossible_positions() {
    assert_all_report(&[
        (
            "P3k3/8/8/8/8/8/8/4K3 w - - 0 1",
            FenViolation::PawnOnBackRank,
        ),
        (
            "4k3/8/8/8/8/8/8/p3K3 w - - 0 1",
            FenViolation::PawnOnBackRank,
        ),
        (
            "4k3/8/8/8/8/P7/PPPPPPPP/4K3 w - - 0 1",
            FenViolation::PawnCount(Side::WHITE, 9),
        ),
        (
            "4k3/pppppppp/p7/8/8/8/8/4K3 b - - 0 1",
            FenViolation::PawnCount(Side::BLACK, 9),
        ),
        (
            "4k3/8/8/8/QQQQQQQQ/QQQQQQQQ/8/4K3 w - - 0 1",
            FenViolation::PieceCount(Side::WHITE, 17),
        ),
        (
            "4k3/8/qqqqqqqq/qqqqqqqq/8/8/8/4K3 b - - 0 1",
            FenViolation::PieceCount(Side::BLACK, 17),
        ),
        ("8/8/8/8/8/8/8/Kk6 w - - 0 1", FenViolation::OpponentInCheck),
        (
            "4k3/8/8/8/8/8/8/4R1K1 w - - 0 1",
            FenViolation::OpponentInCheck,
        ),
        (
            "4k3/8/8/8/8/8/8/4K2r b - - 0 1",
            FenViolation::OpponentInCheck,
        ),
    ]);
}

#[test]
fn reports_invalid_side_to_move() {
    assert_all_report(&[
        (
            "4k3/8/8/8/8/8/8/4K3 W - - 0 1",
            FenViolation::InvalidSideToMove,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 B - - 0 1",
            FenViolation::InvalidSideToMove,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 white - - 0 1",
            FenViolation::InvalidSideToMove,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 x - - 0 1",
            FenViolation::InvalidSideToMove,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 - - - 0 1",
            FenViolation::InvalidSideToMove,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 wb - - 0 1",
            FenViolation::InvalidSideToMove,
        ),
    ]);
}

#[test]
fn reports_malformed_castling_rights() {
    assert_all_report(&[
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w X - 0 1",
            FenViolation::InvalidCastlingRights,
        ),
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkqX - 0 1",
            FenViolation::InvalidCastlingRights,
        ),
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w 1 - 0 1",
            FenViolation::InvalidCastlingRights,
        ),
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w KQ- - 0 1",
            FenViolation::InvalidCastlingRights,
        ),
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w KK - 0 1",
            FenViolation::InvalidCastlingRights,
        ),
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w KQAkq - 0 1",
            FenViolation::InvalidCastlingRights,
        ),
    ]);
}

#[test]
fn reports_unavailable_castling_rights() {
    assert_all_report(&[
        (
            "4k3/8/8/8/8/8/8/4K3 w KQkq - 0 1",
            FenViolation::UnavailableCastlingRight('K'),
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w KQkq - 0 1",
            FenViolation::UnavailableCastlingRight('q'),
        ),
        (
            "r3k2r/8/8/8/8/8/4K3/R6R w KQ - 0 1",
            FenViolation::UnavailableCastlingRight('K'),
        ),
        (
            "r3k2r/8/8/8/8/8/8/4K3 w K - 0 1",
            FenViolation::UnavailableCastlingRight('K'),
        ),
        (
            "r3k2r/8/8/8/8/8/8/R3K2R w G - 0 1",
            FenViolation::UnavailableCastlingRight('G'),
        ),
        (
            "4k3/8/8/8/8/8/8/R3K2R w KQq - 0 1",
            FenViolation::UnavailableCastlingRight('q'),
        ),
    ]);
}

#[test]
fn reports_malformed_en_passant() {
    assert_all_report(&[
        (
            "4k3/8/8/8/8/8/8/4K3 w - e4 0 1",
            FenViolation::InvalidEnPassant,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - e5 0 1",
            FenViolation::InvalidEnPassant,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - e 0 1",
            FenViolation::InvalidEnPassant,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - E6 0 1",
            FenViolation::InvalidEnPassant,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - i6 0 1",
            FenViolation::InvalidEnPassant,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - e66 0 1",
            FenViolation::InvalidEnPassant,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - 66 0 1",
            FenViolation::InvalidEnPassant,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - -- 0 1",
            FenViolation::InvalidEnPassant,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - e0 0 1",
            FenViolation::InvalidEnPassant,
        ),
    ]);
}

#[test]
fn reports_inconsistent_en_passant() {
    assert_all_report(&[
        (
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e3 0 2",
            FenViolation::InvalidEnPassant,
        ),
        (
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e6 0 1",
            FenViolation::InvalidEnPassant,
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq e3 0 1",
            FenViolation::ImpossibleEnPassant,
        ),
        (
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPPPPPP/RNBQKBNR b KQkq e3 0 1",
            FenViolation::ImpossibleEnPassant,
        ),
        (
            "rnbqkbnr/pppppppp/8/4p3/8/8/PPPPPPPP/RNBQKBNR w KQkq e6 0 2",
            FenViolation::ImpossibleEnPassant,
        ),
    ]);
}

#[test]
fn reports_invalid_move_counters() {
    assert_all_report(&[
        (
            "4k3/8/8/8/8/8/8/4K3 w - - -1 1",
            FenViolation::InvalidHalfMoveClock,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - - abc 1",
            FenViolation::InvalidHalfMoveClock,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - - 1.5 1",
            FenViolation::InvalidHalfMoveClock,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - - 256 1",
            FenViolation::InvalidHalfMoveClock,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - - 0 abc",
            FenViolation::InvalidFullMoveNumber,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - - 0 -5",
            FenViolation::InvalidFullMoveNumber,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - - 0 70000",
            FenViolation::InvalidFullMoveNumber,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - - 0 1.0",
            FenViolation::InvalidFullMoveNumber,
        ),
        (
            "4k3/8/8/8/8/8/8/4K3 w - - 0 0",
            FenViolation::InvalidFullMoveNumber,
        ),
    ]);
}

#[test]
fn reports_every_violation_at_once() {
    assert_eq!(
        FEN::validate_fen("rnbqkbnr/pppppppp/8/8/8/PPPPPPPP/RNBQKBNR x KQkq e4 abc 1"),
        vec![
            FenViolation::WrongRankCount(7),
            FenViolation::InvalidSideToMove,
            FenViolation::InvalidEnPassant,
            FenViolation::InvalidHalfMoveClock,
        ]
    );

    assert_eq!(
        FEN::validate_fen("rnbq1bnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKKNR w - z9 0 -1"),
        vec![
            FenViolation::KingCount(Side::WHITE, 2),
            FenViolation::KingCount(Side::BLACK, 0),
            FenViolation::InvalidEnPassant,
            FenViolation::InvalidFullMoveNumber,
        ]
    );
}

#[test]
fn validation_never_panics() {
    let panicked: Vec<String> = fuzzed_fens(20_000)
        .into_iter()
        .filter(|fen| panic::catch_unwind(|| FEN::validate_fen(fen)).is_err())
        .collect();

    assert!(
        panicked.is_empty(),
        "validate_fen panicked on:\n{}",
        panicked.join("\n")
    );
}

#[test]
fn accepted_fens_load_cleanly() {
    let mut accepted = 0;
    let mut malformed = Vec::new();
    let mut panicked = Vec::new();
    let mut corrupted = Vec::new();

    for fen in fuzzed_fens(20_000) {
        if !FEN::validate_fen(&fen).is_empty() {
            continue;
        }

        accepted += 1;

        let placement = fen.split_whitespace().next().unwrap_or_default();
        let Some(expected) = expand_placement(placement) else {
            malformed.push(fen);
            continue;
        };

        let loaded = panic::catch_unwind(|| {
            FEN::from(&ChessBoard::from(&FEN::from(fen.as_str()))).to_string()
        });
        match loaded {
            Err(_) => panicked.push(fen),
            Ok(output) => {
                let output_placement = output.split_whitespace().next().unwrap_or_default();
                if expand_placement(output_placement) != Some(expected) {
                    corrupted.push(format!("{fen}  ->  {output}"));
                }
            }
        }
    }

    let report = |name: &str, fens: &[String]| {
        format!(
            "{} {name} (first 10):\n{}",
            fens.len(),
            fens.iter().take(10).cloned().collect::<Vec<_>>().join("\n")
        )
    };

    assert!(
        accepted >= 500,
        "only {accepted} fuzzed FENs were accepted, too few to check loading"
    );

    assert!(
        malformed.is_empty() && panicked.is_empty() && corrupted.is_empty(),
        "accepted FENs that don't load cleanly:\n{}\n\n{}\n\n{}",
        report("with malformed piece placement", &malformed),
        report("that panicked while loading", &panicked),
        report("whose pieces changed after loading", &corrupted),
    );
}
