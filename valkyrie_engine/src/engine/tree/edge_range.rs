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

use super::components::Clear;

const FIRST_BITS: u32 = 40;
const FIRST_MASK: u64 = (1 << FIRST_BITS) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EdgeRange {
    first: u64,
    count: u32,
}

impl EdgeRange {
    pub const EMPTY: Self = Self { first: 0, count: 0 };
    pub const MAX_SLOTS: usize = FIRST_MASK as usize;
    pub const MAX_COUNT: usize = (1 << (u64::BITS - FIRST_BITS)) - 1;

    #[inline]
    pub fn new(first: usize, count: usize) -> Self {
        debug_assert!(first <= Self::MAX_SLOTS && count <= Self::MAX_COUNT);
        Self {
            first: first as u64,
            count: count as u32,
        }
    }

    #[inline]
    pub fn first(&self) -> usize {
        self.first as usize
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.count as usize
    }

    fn pack(self) -> u64 {
        self.first | (u64::from(self.count) << FIRST_BITS)
    }

    fn unpack(value: u64) -> Self {
        Self {
            first: value & FIRST_MASK,
            count: (value >> FIRST_BITS) as u32,
        }
    }
}

#[derive(Debug, Default)]
pub struct EdgeRangeStore(AtomicU64);

impl EdgeRangeStore {
    #[inline]
    pub fn load(&self) -> EdgeRange {
        EdgeRange::unpack(self.0.load(Ordering::Acquire))
    }

    #[inline]
    pub fn store(&self, range: EdgeRange) {
        self.0.store(range.pack(), Ordering::Release);
    }
}

impl Clear for EdgeRangeStore {
    #[inline]
    fn clear(&self) {
        self.store(EdgeRange::EMPTY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LARGEST_HASH_BYTES: usize = 512 * 1024 * 1024 * 1024;
    const SMALLEST_EDGE_BYTES: usize = 8;

    #[test]
    fn ranges_address_a_512_gb_tree() {
        let last_slot = LARGEST_HASH_BYTES / SMALLEST_EDGE_BYTES;
        let store = EdgeRangeStore::default();

        assert!(last_slot <= EdgeRange::MAX_SLOTS);

        store.store(EdgeRange::new(last_slot, 218));

        assert_eq!(store.load().first(), last_slot);
        assert_eq!(store.load().count(), 218);
    }

    #[test]
    fn empty_is_the_default() {
        assert_eq!(EdgeRangeStore::default().load(), EdgeRange::EMPTY);
    }
}
