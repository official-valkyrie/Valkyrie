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

pub mod components;
pub mod edge_storage;
pub mod edges;
pub mod nodes;

pub(crate) mod edge_index;
pub(crate) mod node_index;

use std::{
    ops::Index,
    sync::atomic::{AtomicUsize, Ordering},
};

use components::{HasChild, NodeType};

pub use edge_index::EdgeIndex;
pub use edge_storage::EdgesStore;
pub use node_index::{AtomicNodeIndex, NodeIndex};

const AVERAGE_EDGES_PER_NODE: usize = 30;

#[derive(Debug)]
pub struct Tree<N> {
    nodes: Vec<N>,
    root_idx: AtomicNodeIndex,
    size_in_mb: usize,
    current_len: AtomicUsize,
}

impl<N: NodeType> Tree<N> {
    pub fn new(size_in_mb: usize) -> Self {
        let bytes = size_in_mb * 1024 * 1024;
        let node_size = size_of::<N>() + size_of::<N::Edge>() * AVERAGE_EDGES_PER_NODE;
        let size = bytes / node_size;

        let tree = Self {
            nodes: std::iter::repeat_with(N::default).take(size).collect(),
            root_idx: AtomicNodeIndex::new(NodeIndex::NULL),
            size_in_mb,
            current_len: AtomicUsize::new(0),
        };

        tree.clear();
        tree
    }

    pub fn resize(&mut self, size_in_mb: usize) {
        *self = Self::new(size_in_mb)
    }

    pub fn clear(&self) {
        self.current_len.store(0, Ordering::Relaxed);
        self.root_idx
            .store(self.create_node().unwrap_or(NodeIndex::NULL));
    }

    pub fn root_index(&self) -> NodeIndex {
        self.root_idx.load()
    }

    pub fn root_node(&self) -> &N {
        &self[self.root_index()]
    }

    pub fn len(&self) -> usize {
        self.current_len.load(Ordering::Relaxed)
    }

    pub fn capacity(&self) -> usize {
        self.nodes.len()
    }

    pub fn size_in_mb(&self) -> usize {
        self.size_in_mb
    }

    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity()
    }

    pub fn create_node(&self) -> Option<NodeIndex> {
        let node_idx = self
            .current_len
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |len| {
                (len < self.capacity()).then_some(len + 1)
            })
            .ok()?;

        self.nodes[node_idx].clear();
        Some(NodeIndex::new(node_idx))
    }

    pub fn create_child(&self, node_idx: NodeIndex, edge_idx: EdgeIndex) -> Option<NodeIndex> {
        let child_idx = self.create_node()?;
        self[node_idx].edges()[edge_idx].set_child(child_idx);
        Some(child_idx)
    }

    pub fn get_or_create_child(
        &self,
        node_idx: NodeIndex,
        edge_idx: EdgeIndex,
    ) -> Option<NodeIndex> {
        let child_idx = self[node_idx].edges()[edge_idx].child();

        if child_idx == NodeIndex::NULL {
            self.create_child(node_idx, edge_idx)
        } else {
            Some(child_idx)
        }
    }
}

impl<N> Index<NodeIndex> for Tree<N> {
    type Output = N;

    fn index(&self, index: NodeIndex) -> &Self::Output {
        self.nodes.get(index.raw() as usize).unwrap()
    }
}
