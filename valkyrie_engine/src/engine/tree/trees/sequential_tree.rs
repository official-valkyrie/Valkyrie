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
    components::{HasChild, NodeType, link_child},
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

    fn resize(&mut self, size_in_mb: usize) {
        self.arena = Arena::new(0);
        self.arena = Arena::new(size_in_mb * BYTES_PER_MB);
        self.clear();
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

    fn edge_index(&self, node: &N, slot: usize) -> EdgeIndex {
        self.arena.edge_index(node.edge_range(), slot)
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
        node.edge_range_store().publish(range);

        Ok(())
    }

    fn get_or_create_child(&self, edge: EdgeIndex, _key: NodeKey) -> Result<Child, TreeFull> {
        let edge = &self[edge];
        if edge.has_child() {
            return Ok(Child::Found(edge.child()));
        }

        let child_idx = self.arena.alloc_node().ok_or(TreeFull)?;
        link_child(edge, child_idx);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::tree::{edges::AvgScoreEdge, nodes::ClassicNode};

    type TestTree = SequentialTree<ClassicNode<AvgScoreEdge>>;

    #[test]
    fn a_node_is_expanded_once() {
        let tree = TestTree::new(1);
        let root = tree.root_node();

        tree.expand(root, 3, |_| {}).unwrap();
        tree.expand(root, 5, |_| panic!("expanded twice")).unwrap();

        assert_eq!(root.edge_count(), 3);
    }

    #[test]
    fn a_failed_expansion_can_be_retried() {
        let tree = TestTree::new(1);
        let root = tree.root_node();

        assert_eq!(tree.expand(root, tree.total_bytes(), |_| {}), Err(TreeFull));
        assert_eq!(tree.expand(root, 3, |_| {}), Ok(()));
        assert_eq!(root.edge_count(), 3);
    }

    #[test]
    fn a_child_is_created_once() {
        let tree = TestTree::new(1);
        tree.expand(tree.root_node(), 1, |_| {}).unwrap();
        let edge = tree.edge_index(tree.root_node(), 0);

        let created = tree.get_or_create_child(edge, NodeKey::new(0, 0)).unwrap();
        let found = tree.get_or_create_child(edge, NodeKey::new(0, 0)).unwrap();

        assert_eq!(created, Child::Created(found.index()));
        assert_eq!(found, Child::Found(tree[edge].child()));
    }
}
