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

use std::sync::atomic::{
    AtomicU8,
    Ordering,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchState {
    Idle,
    Running,
    Ended,
    Interrupted,
}

#[derive(Debug)]
pub struct AtomicSearchState(AtomicU8);

impl Default for AtomicSearchState {
    fn default() -> Self {
        Self(AtomicU8::new(SearchState::Idle as u8))
    }
}

impl AtomicSearchState {
    #[inline]
    pub fn is(&self, state: SearchState) -> bool {
        self.0.load(Ordering::Relaxed) == state as u8
    }

    #[inline]
    pub fn set(&self, state: SearchState) {
        self.0.store(state as u8, Ordering::Relaxed)
    }

    #[inline]
    pub fn transition(&self, from: SearchState, to: SearchState) {
        let _ = self.0.compare_exchange(from as u8, to as u8, Ordering::Relaxed, Ordering::Relaxed);
    }
}
