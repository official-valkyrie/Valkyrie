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
    alloc::{self, Layout},
    marker::PhantomData,
    mem::{align_of, needs_drop, size_of},
    ptr::NonNull,
    slice,
    sync::atomic::{AtomicUsize, Ordering},
};

use super::{EdgeIndex, EdgeRange, NodeIndex, components::NodeType};

const ALIGNMENT: usize = 64;
const PAGE_SIZE: usize = 4096;

#[derive(Debug)]
pub struct Arena<N: NodeType> {
    base: NonNull<u8>,
    layout: Layout,
    edge_end: usize,
    used: AtomicUsize,
    nodes: AtomicUsize,
    edges: AtomicUsize,
    _nodes: PhantomData<N>,
}

unsafe impl<N: NodeType> Send for Arena<N> {}
unsafe impl<N: NodeType> Sync for Arena<N> {}

impl<N: NodeType> Arena<N> {
    const NODE_SIZE: usize = size_of::<N>();
    const EDGE_SIZE: usize = size_of::<N::Edge>();

    pub fn new(bytes: usize) -> Self {
        const {
            assert!(!needs_drop::<N>() && !needs_drop::<N::Edge>());
            assert!(size_of::<N>() > 0 && size_of::<N::Edge>() > 0);
            assert!(align_of::<N>() <= ALIGNMENT && align_of::<N::Edge>() <= ALIGNMENT);
        }

        let layout = Layout::from_size_align(bytes.max(ALIGNMENT), ALIGNMENT)
            .expect("tree size does not fit in memory");
        let base = NonNull::new(unsafe { alloc::alloc(layout) })
            .unwrap_or_else(|| alloc::handle_alloc_error(layout));

        for offset in (0..layout.size()).step_by(PAGE_SIZE) {
            unsafe { base.as_ptr().add(offset).write(0) };
        }

        let usable = layout.size().min(EdgeRange::MAX_SLOTS * Self::EDGE_SIZE);

        Self {
            base,
            layout,
            edge_end: usable - usable % align_of::<N::Edge>(),
            used: AtomicUsize::new(0),
            nodes: AtomicUsize::new(0),
            edges: AtomicUsize::new(0),
            _nodes: PhantomData,
        }
    }

    pub fn reset(&mut self) {
        *self.used.get_mut() = 0;
        *self.nodes.get_mut() = 0;
        *self.edges.get_mut() = 0;
    }

    pub fn total_bytes(&self) -> usize {
        self.edge_end
    }

    pub fn used_bytes(&self) -> usize {
        self.used.load(Ordering::Relaxed)
    }

    pub fn node_count(&self) -> usize {
        self.nodes.load(Ordering::Relaxed)
    }

    pub fn edge_count(&self) -> usize {
        self.edges.load(Ordering::Relaxed)
    }

    pub fn alloc_node(&self) -> Option<NodeIndex> {
        self.reserve(Self::NODE_SIZE)?;
        let node = self.nodes.fetch_add(1, Ordering::Relaxed);

        unsafe { self.node_ptr(node).write(N::default()) };

        Some(NodeIndex::new(node))
    }

    pub fn alloc_edges(&self, count: usize) -> Option<EdgeRange> {
        self.reserve(count * Self::EDGE_SIZE)?;
        let first = self.edges.fetch_add(count, Ordering::Relaxed) + count;
        let block = unsafe { self.edge_ptr(first) };

        for slot in 0..count {
            unsafe { block.add(slot).write(N::Edge::default()) };
        }

        Some(EdgeRange::new(first, count))
    }

    pub fn node(&self, index: NodeIndex) -> &N {
        assert!(
            index.raw() < self.node_count(),
            "node {index:?} was never allocated"
        );
        unsafe { &*self.node_ptr(index.raw()) }
    }

    pub fn edge(&self, index: EdgeIndex) -> &N::Edge {
        assert!(
            index.raw() < self.edge_count(),
            "edge {index:?} was never allocated"
        );
        unsafe { &*self.edge_ptr(index.raw() + 1) }
    }

    pub fn edge_index(&self, range: EdgeRange, slot: usize) -> EdgeIndex {
        debug_assert!(slot < range.count());
        EdgeIndex::new(range.first() - 1 - slot)
    }

    pub fn edges(&self, range: EdgeRange) -> &[N::Edge] {
        if range.count() == 0 {
            return &[];
        }

        unsafe { slice::from_raw_parts(self.edge_ptr(range.first()), range.count()) }
    }

    fn reserve(&self, bytes: usize) -> Option<()> {
        self.used
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
                used.checked_add(bytes)
                    .filter(|&total| total <= self.edge_end)
            })
            .ok()
            .map(|_| ())
    }

    unsafe fn node_ptr(&self, node: usize) -> *mut N {
        unsafe { self.base.as_ptr().add(node * Self::NODE_SIZE).cast() }
    }

    unsafe fn edge_ptr(&self, first: usize) -> *mut N::Edge {
        unsafe {
            self.base
                .as_ptr()
                .add(self.edge_end - first * Self::EDGE_SIZE)
                .cast()
        }
    }
}

impl<N: NodeType> Drop for Arena<N> {
    fn drop(&mut self) {
        unsafe { alloc::dealloc(self.base.as_ptr(), self.layout) };
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread};

    use super::*;
    use crate::engine::tree::{
        components::{HasChild, HasVisits, link_child},
        edges::AvgScoreEdge,
        nodes::ClassicNode,
    };

    type TestArena = Arena<ClassicNode<AvgScoreEdge>>;

    const NODE: usize = size_of::<ClassicNode<AvgScoreEdge>>();
    const EDGE: usize = size_of::<AvgScoreEdge>();

    fn free(arena: &TestArena) -> usize {
        arena.total_bytes() - arena.used_bytes()
    }

    #[test]
    fn free_space_is_the_gap_between_both_ends() {
        let arena = TestArena::new(1 << 16);
        let total = arena.total_bytes();

        arena.alloc_node().unwrap();
        arena.alloc_edges(5).unwrap();
        arena.alloc_node().unwrap();

        assert_eq!(arena.used_bytes(), 2 * NODE + 5 * EDGE);
        assert_eq!(free(&arena), total - 2 * NODE - 5 * EDGE);
    }

    #[test]
    fn allocation_fails_exactly_when_it_would_overlap() {
        let arena = TestArena::new(1 << 12);

        while arena.alloc_node().is_some() {}
        assert!(free(&arena) < NODE);
        assert_eq!(arena.alloc_edges(free(&arena) / EDGE + 1), None);

        let fitting = free(&arena) / EDGE;
        assert!(arena.alloc_edges(fitting).is_some());
        assert!(free(&arena) < EDGE);
    }

    #[test]
    fn nodes_and_edges_never_overwrite_each_other() {
        let arena = TestArena::new(1 << 14);
        let mut blocks = Vec::new();

        while let (Some(node), Some(range)) = (arena.alloc_node(), arena.alloc_edges(3)) {
            arena.node(node).add_visits(node.raw() as u64 + 1);

            for (slot, edge) in arena.edges(range).iter().enumerate() {
                link_child(edge, NodeIndex::new(node.raw() * 10 + slot));
            }

            blocks.push((node, range));
        }

        assert!(blocks.len() > 3);

        for (node, range) in blocks {
            assert_eq!(arena.node(node).visits(), node.raw() as u64 + 1);

            for (slot, edge) in arena.edges(range).iter().enumerate() {
                assert_eq!(edge.child(), NodeIndex::new(node.raw() * 10 + slot));
            }
        }
    }

    #[test]
    fn fresh_slots_start_from_defaults_after_reset() {
        let mut arena = TestArena::new(1 << 12);
        let node = arena.alloc_node().unwrap();
        let range = arena.alloc_edges(2).unwrap();

        arena.node(node).add_visits(7);
        link_child(&arena.edges(range)[0], NodeIndex::new(3));
        arena.reset();

        assert_eq!(arena.used_bytes(), 0);

        let node = arena.alloc_node().unwrap();
        let range = arena.alloc_edges(2).unwrap();

        assert_eq!(arena.node(node).visits(), 0);
        assert_eq!(arena.edges(range)[0].child(), NodeIndex::NULL);
    }

    #[test]
    #[should_panic(expected = "never allocated")]
    fn unallocated_nodes_cannot_be_read() {
        let arena = TestArena::new(1 << 12);
        arena.node(NodeIndex::new(0));
    }

    #[test]
    #[should_panic(expected = "never allocated")]
    fn unallocated_edges_cannot_be_read() {
        let arena = TestArena::new(1 << 12);
        arena.edge(EdgeIndex::new(0));
    }

    #[test]
    fn edge_indices_resolve_without_their_node() {
        let arena = TestArena::new(1 << 12);
        let first = arena.alloc_edges(3).unwrap();
        let second = arena.alloc_edges(2).unwrap();

        for range in [first, second] {
            for slot in 0..range.count() {
                let index = arena.edge_index(range, slot);
                assert!(std::ptr::eq(arena.edge(index), &arena.edges(range)[slot]));
            }
        }
    }

    #[test]
    fn concurrent_allocation_never_hands_out_a_slot_twice() {
        let arena = Arc::new(TestArena::new(1 << 18));

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let arena = Arc::clone(&arena);
                thread::spawn(move || {
                    let mut taken = Vec::new();
                    while let Some(node) = arena.alloc_node() {
                        let Some(range) = arena.alloc_edges(2) else {
                            break;
                        };
                        taken.push((node.raw(), range.first()));
                    }
                    taken
                })
            })
            .collect();

        let taken: Vec<(usize, usize)> = handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect();
        let mut nodes: Vec<usize> = taken.iter().map(|&(node, _)| node).collect();
        let mut edge_blocks: Vec<usize> = taken.iter().map(|&(_, first)| first).collect();
        nodes.sort_unstable();
        nodes.dedup();
        edge_blocks.sort_unstable();
        edge_blocks.dedup();

        assert_eq!(nodes.len(), taken.len());
        assert_eq!(edge_blocks.len(), taken.len());
        assert!(arena.used_bytes() <= arena.total_bytes());
    }
}
