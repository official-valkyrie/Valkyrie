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

use valkyrie_chess::FEN;

#[test]
fn accepts_legal_positions() {
    let mut fens = vec![
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1".to_string(),
    ];

    for path in ["./tests/standard.epd", "./tests/fischer.epd"] {
        let contents = std::fs::read_to_string(path).unwrap();
        fens.extend(
            contents
                .lines()
                .filter_map(|line| line.split(';').next())
                .map(|fen| fen.trim().to_string()),
        );
    }

    for fen in fens {
        assert!(FEN::validate_fen(&fen).is_empty(), "{fen}");
    }
}

#[test]
fn rejects_illegal_positions() {
    for fen in [
        "",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
        "rnbqkbnr/pppppppp/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/ppppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBXKBNR w KQkq - 0 1",
        "rnbqqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1",
        "rnbkkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQ - 0 1",
        "P3k3/8/8/8/8/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/P7/PPPPPPPP/4K3 w - - 0 1",
        "8/8/8/8/8/8/8/Kk6 w - - 0 1",
        "4k3/8/8/8/8/8/8/4R1K1 w - - 0 1",
        "4k3/8/8/8/8/8/8/4K3 x - - 0 1",
        "4k3/8/8/8/8/8/8/4K3 w KQkq - 0 1",
        "4k3/8/8/8/8/8/8/4K3 w - e5 0 1",
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq e3 0 1",
        "4k3/8/8/8/8/8/8/4K3 w - - -1 1",
        "4k3/8/8/8/8/8/8/4K3 w - - 0 0",
    ] {
        assert!(!FEN::validate_fen(fen).is_empty(), "{fen}");
    }
}
