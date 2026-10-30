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

use valkyrie_chess::{ChessPosition, Piece};

use crate::{engine::builder::SimulationStrategy, prelude::*};

const PIECE_VALUES: [i32; 5] = [100, 300, 330, 500, 900];

#[derive(Debug)]
pub struct MaterialEvaluation;

impl Strategy for MaterialEvaluation {
    type Params = EmptyParams;
}

impl<C: EngineConfig> SimulationStrategy<C> for MaterialEvaluation {
    type Output = WinChance;

    fn execute(
        _raw_eval: &[f32],
        position: &ChessPosition,
        _params: &Self::Params,
        _engine: &Engine<C>,
    ) -> Self::Output {
        let board = position.board();
        let side = board.side();

        let mut score = 0i32;
        for (index, value) in PIECE_VALUES.iter().enumerate() {
            let piece = Piece::from(index);
            let own = board.piece_mask_for_side(piece, side).pop_count() as i32;
            let opponent = board.piece_mask_for_side(piece, side.flipped()).pop_count() as i32;

            score += (own - opponent) * value;
        }

        WinChance::from_cp(score as f32)
    }
}
