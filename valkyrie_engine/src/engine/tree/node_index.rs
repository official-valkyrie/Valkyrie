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

use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeIndex(usize);

impl NodeIndex {
    pub const NULL: Self = Self(usize::MAX);

    #[inline]
    pub fn new(value: usize) -> Self {
        Self(value)
    }

    #[inline]
    pub fn raw(&self) -> usize {
        self.0
    }
}

impl Default for NodeIndex {
    fn default() -> Self {
        Self::NULL
    }
}

impl From<usize> for NodeIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<NodeIndex> for usize {
    #[inline]
    fn from(value: NodeIndex) -> Self {
        value.0
    }
}

#[derive(Debug)]
pub struct AtomicNodeIndex(AtomicUsize);

impl AtomicNodeIndex {
    #[inline]
    pub fn new(index: NodeIndex) -> Self {
        Self(AtomicUsize::new(index.raw()))
    }

    #[inline]
    pub fn null() -> Self {
        Self(AtomicUsize::new(NodeIndex::NULL.raw()))
    }

    #[inline]
    pub fn load(&self) -> NodeIndex {
        NodeIndex::new(self.0.load(Ordering::Relaxed))
    }

    #[inline]
    pub fn store(&self, index: NodeIndex) {
        self.0.store(index.raw(), Ordering::Relaxed);
    }
}

impl Default for AtomicNodeIndex {
    fn default() -> Self {
        Self::null()
    }
}
