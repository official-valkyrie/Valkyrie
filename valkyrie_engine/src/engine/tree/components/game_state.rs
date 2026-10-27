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

use std::sync::atomic::{AtomicU16, Ordering};

use valkyrie_chess::GameState;

use super::{Clear, HasComponent};

const PAYLOAD_OFFSET: u16 = 14;
const PAYLOAD_MASK: u16 = 0x3FFF;

#[derive(Debug, Default)]
pub struct GameStateStore(AtomicU16);

impl Clone for GameStateStore {
    #[inline]
    fn clone(&self) -> Self {
        Self(AtomicU16::new(self.0.load(Ordering::Relaxed)))
    }
}

impl Clear for GameStateStore {
    #[inline]
    fn clear(&self) {
        self.0.store(Self::pack(GameState::Ongoing), Ordering::Relaxed);
    }
}

impl GameStateStore {
    #[inline]
    fn pack(state: GameState) -> u16 {
        match state {
            GameState::Ongoing => 0,
            GameState::Won(x) => 1 << PAYLOAD_OFFSET | x.min(PAYLOAD_MASK),
            GameState::Lost(x) => 2 << PAYLOAD_OFFSET | x.min(PAYLOAD_MASK),
            GameState::Drew => 3 << PAYLOAD_OFFSET,
        }
    }
}

pub trait HasGameState {
    fn game_state(&self) -> GameState;
    fn set_game_state(&self, state: GameState);

    #[inline]
    fn is_terminal(&self) -> bool {
        self.game_state() != GameState::Ongoing
    }
}

impl<T: HasComponent<GameStateStore>> HasGameState for T {
    #[inline]
    fn game_state(&self) -> GameState {
        let value = self.component().0.load(Ordering::Relaxed);
        let tag = value >> PAYLOAD_OFFSET;
        let payload = value & PAYLOAD_MASK;
        match tag {
            0 => GameState::Ongoing,
            1 => GameState::Won(payload),
            2 => GameState::Lost(payload),
            3 => GameState::Drew,
            _ => unreachable!(),
        }
    }

    #[inline]
    fn set_game_state(&self, state: GameState) {
        self.component()
            .0
            .store(GameStateStore::pack(state), Ordering::Relaxed);
    }
}
