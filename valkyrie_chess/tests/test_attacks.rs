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

use valkyrie_chess::{Attacks, Bitboard, Square};

const ROOK_STEPS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
const BISHOP_STEPS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

fn walk_rays(rank: u8, file: u8, occupancy: u64, steps: &[(i8, i8); 4]) -> u64 {
    let mut attacks = 0u64;

    for &(rank_step, file_step) in steps {
        let (mut r, mut f) = (rank as i8 + rank_step, file as i8 + file_step);

        while (0..8).contains(&r) && (0..8).contains(&f) {
            let bit = 1u64 << (r * 8 + f);
            attacks |= bit;

            if occupancy & bit != 0 {
                break;
            }

            r += rank_step;
            f += file_step;
        }
    }

    attacks
}

#[test]
fn slider_attacks_match_ray_walk() {
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    for rank in 0..8 {
        for file in 0..8 {
            let square = Square::from_coords(rank, file);

            for round in 0..2000 {
                let occupancy = match round % 3 {
                    0 => next(),
                    1 => next() & next(),
                    _ => next() & next() & next(),
                };

                assert_eq!(
                    u64::from(Attacks::get_rook_attacks(square, Bitboard::from(occupancy))),
                    walk_rays(rank, file, occupancy, &ROOK_STEPS),
                    "rook on {square} with occupancy {occupancy:#x}"
                );
                assert_eq!(
                    u64::from(Attacks::get_bishop_attacks(
                        square,
                        Bitboard::from(occupancy)
                    )),
                    walk_rays(rank, file, occupancy, &BISHOP_STEPS),
                    "bishop on {square} with occupancy {occupancy:#x}"
                );
            }
        }
    }
}
