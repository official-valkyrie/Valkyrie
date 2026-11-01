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

use std::sync::atomic::{AtomicU64, Ordering};

use super::{Clear, HasComponent};

#[derive(Debug, Default)]
pub struct ScoreSumStore(AtomicU64);

impl Clear for ScoreSumStore {
    #[inline]
    fn clear(&self) {
        self.0.store(0, Ordering::Relaxed);
    }
}

pub trait HasScoreSum {
    fn total_score(&self) -> f64;
    fn set_score(&self, value: f64);
    fn add_score(&self, value: f64);
}

impl<T: HasComponent<ScoreSumStore>> HasScoreSum for T {
    #[inline]
    fn total_score(&self) -> f64 {
        f64::from_bits(self.component().0.load(Ordering::Relaxed))
    }

    #[inline]
    fn set_score(&self, value: f64) {
        self.component().0.store(value.to_bits(), Ordering::Relaxed);
    }

    #[inline]
    fn add_score(&self, value: f64) {
        let score = &self.component().0;
        loop {
            let current = score.load(Ordering::Relaxed);
            let new = (f64::from_bits(current) + value).to_bits();
            if score
                .compare_exchange_weak(current, new, Ordering::Relaxed, Ordering::Relaxed)
                .is_ok()
            {
                break;
            }
        }
    }
}
