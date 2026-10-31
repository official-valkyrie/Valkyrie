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

use std::ops::Index;

use crate::engine::tree::{
    Child, EdgeIndex, EdgeOf, NodeIndex, NodeKey, Tree, TreeFull,
    arena::Arena,
    components::{HasChild, NodeType},
};

const BYTES_PER_MB: usize = 1024 * 1024;

#[derive(Debug)]
pub struct SequentialTree<N: NodeType> {
    arena: Arena<N>,
    root_idx: NodeIndex,
}

impl<N: NodeType> Tree for SequentialTree<N> {
    type Node = N;

    fn new(size_in_mb: usize) -> Self {
        let mut tree = Self {
            arena: Arena::new(size_in_mb * BYTES_PER_MB),
            root_idx: NodeIndex::NULL,
        };

        tree.clear();
        tree
    }

    fn clear(&mut self) {
        self.arena.reset();
        self.root_idx = self.arena.alloc_node().unwrap_or(NodeIndex::NULL);
    }

    fn root_index(&self) -> NodeIndex {
        self.root_idx
    }

    fn used_bytes(&self) -> usize {
        self.arena.used_bytes()
    }

    fn total_bytes(&self) -> usize {
        self.arena.total_bytes()
    }

    fn edges(&self, node: &N) -> &[EdgeOf<Self>] {
        self.arena.edges(node.edge_range())
    }

    fn edge_index(&self, node: &N, position: usize) -> EdgeIndex {
        self.arena.edge_index(node.edge_range(), position)
    }

    fn expand(
        &self,
        node: &N,
        count: usize,
        fill: impl FnOnce(&[EdgeOf<Self>]),
    ) -> Result<(), TreeFull> {
        if node.edge_count() > 0 {
            return Ok(());
        }

        let range = self.arena.alloc_edges(count).ok_or(TreeFull)?;
        fill(self.arena.edges(range));
        node.set_edge_range(range);

        Ok(())
    }

    fn get_or_create_child(&self, edge: EdgeIndex, _key: NodeKey) -> Result<Child, TreeFull> {
        let edge = &self[edge];
        if edge.has_child() {
            return Ok(Child::Found(edge.child()));
        }

        let child_idx = self.arena.alloc_node().ok_or(TreeFull)?;
        edge.set_child(child_idx);
        Ok(Child::Created(child_idx))
    }
}

impl<N: NodeType> Index<NodeIndex> for SequentialTree<N> {
    type Output = N;

    fn index(&self, index: NodeIndex) -> &N {
        self.arena.node(index)
    }
}

impl<N: NodeType> Index<EdgeIndex> for SequentialTree<N> {
    type Output = N::Edge;

    fn index(&self, index: EdgeIndex) -> &N::Edge {
        self.arena.edge(index)
    }
}
