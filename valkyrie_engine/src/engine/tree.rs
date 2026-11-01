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
pub mod edge_range;
pub mod edges;
pub mod node_key;
pub mod nodes;
pub mod trees;

pub(crate) mod arena;
pub(crate) mod edge_index;
pub(crate) mod node_index;

use std::ops::Index;

use components::NodeType;

pub use edge_index::EdgeIndex;
pub use edge_range::{EdgeRange, EdgeRangeStore};
pub use node_index::{AtomicNodeIndex, NodeIndex};
pub use node_key::NodeKey;
pub use trees::SequentialTree;

pub type EdgeOf<T> = <<T as Tree>::Node as NodeType>::Edge;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeFull;

pub trait Tree:
    Index<NodeIndex, Output = Self::Node>
    + Index<EdgeIndex, Output = EdgeOf<Self>>
    + std::fmt::Debug
    + Send
    + Sync
    + 'static
{
    type Node: NodeType;

    fn new(size_in_mb: usize) -> Self;

    fn resize(&mut self, size_in_mb: usize);

    fn clear(&mut self);

    fn reroot(&mut self, _key: NodeKey) {
        self.clear();
    }

    fn root_index(&self) -> NodeIndex;

    fn root_node(&self) -> &Self::Node {
        &self[self.root_index()]
    }

    fn used_bytes(&self) -> usize;

    fn total_bytes(&self) -> usize;

    fn edges(&self, node: &Self::Node) -> &[EdgeOf<Self>];

    fn edge_index(&self, node: &Self::Node, slot: usize) -> EdgeIndex;

    fn expand(
        &self,
        node: &Self::Node,
        count: usize,
        fill: impl FnOnce(&[EdgeOf<Self>]),
    ) -> Result<(), TreeFull>;

    fn get_or_create_child(&self, edge: EdgeIndex, key: NodeKey) -> Result<NodeIndex, TreeFull>;

    #[inline]
    fn node_entered(&self, _node: NodeIndex) {}

    #[inline]
    fn node_exited(&self, _node: NodeIndex) {}
}
