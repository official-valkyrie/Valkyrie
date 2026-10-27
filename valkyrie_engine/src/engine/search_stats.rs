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
    sync::atomic::{
        AtomicU64,
        Ordering,
    },
    time::Instant,
};

#[derive(Debug)]
pub struct SearchStats {
    start_time: Instant,
    max_depth: AtomicU64,
    cumulative_depth: AtomicU64,
    iterations: AtomicU64,
}

impl Default for SearchStats {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchStats {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            max_depth: AtomicU64::new(0),
            cumulative_depth: AtomicU64::new(0),
            iterations: AtomicU64::new(0),
        }
    }

    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    pub fn max_depth(&self) -> u64 {
        self.max_depth.load(Ordering::Relaxed)
    }

    pub fn cumulative_depth(&self) -> u64 {
        self.cumulative_depth.load(Ordering::Relaxed)
    }

    pub fn iterations(&self) -> u64 {
        self.iterations.load(Ordering::Relaxed)
    }

    pub fn avg_depth(&self) -> u64 {
        self.cumulative_depth() / self.iterations().max(1)
    }

    pub fn add_iteration(&self, depth: u64) {
        self.max_depth.fetch_max(depth, Ordering::Relaxed);
        self.cumulative_depth.fetch_add(depth, Ordering::Relaxed);
        self.iterations.fetch_add(1, Ordering::Relaxed);
    }
}
