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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameState {
    #[default]
    Ongoing,
    Won(u16),
    Lost(u16),
    Drew,
}

impl GameState {
    #[inline]
    pub fn for_parent(self) -> Self {
        match self {
            Self::Won(plies) => Self::Lost(plies.saturating_add(1)),
            Self::Lost(plies) => Self::Won(plies.saturating_add(1)),
            state => state,
        }
    }

    #[inline]
    pub fn mate_in(self) -> Option<i32> {
        match self {
            Self::Won(plies) => Some(i32::from(plies.div_ceil(2))),
            Self::Lost(plies) => Some(-i32::from(plies.div_ceil(2))),
            _ => None,
        }
    }
}
