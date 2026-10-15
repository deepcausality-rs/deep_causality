/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::storage::graph_csm::adjacency::Adjacency;
use crate::{EdgeKey, GraphView, Projection, ProjectionView};
use alloc::vec::Vec;

/// Every read answers for the passing nodes and edges; the mode plays no part in a read.
impl<N, W, P> GraphView<N, W> for ProjectionView<'_, N, W, P>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
    P: Projection<N, W>,
{
    fn is_frozen(&self) -> bool {
        true
    }

    fn is_empty(&self) -> bool {
        self.passing_count() == 0
    }

    fn contains_node(&self, index: usize) -> bool {
        self.passes(index)
    }

    fn get_node(&self, index: usize) -> Option<&N> {
        self.graph
            .get_node(index)
            .filter(|node| self.projection.node(node))
    }

    fn number_nodes(&self) -> usize {
        self.passing_count()
    }

    fn contains_edge(&self, a: usize, b: usize) -> bool {
        self.out_edges(a).any(|(target, _)| target == b)
    }

    fn contains_edge_keyed(&self, a: usize, b: usize, key: W::Key) -> bool {
        self.out_edges(a)
            .any(|(target, weight)| target == b && weight.edge_key() == key)
    }

    fn number_edges(&self) -> usize {
        (0..self.slots()).map(|v| self.out_edges(v).count()).sum()
    }

    fn get_all_nodes(&self) -> Vec<&N> {
        (0..self.slots()).filter_map(|v| self.get_node(v)).collect()
    }

    fn get_edges(&self, source: usize) -> Option<Vec<(usize, &W)>> {
        if !self.passes(source) {
            return None;
        }
        Some(self.out_edges(source).collect())
    }

    fn get_last_index(&self) -> Option<usize> {
        (0..self.slots()).rev().find(|&v| self.passes(v))
    }

    fn contains_root_node(&self) -> bool {
        self.get_root_index().is_some()
    }

    fn get_root_node(&self) -> Option<&N> {
        self.get_root_index().and_then(|index| self.get_node(index))
    }

    /// The graph's root, when it passes.
    fn get_root_index(&self) -> Option<usize> {
        self.graph
            .get_root_index()
            .filter(|&index| self.passes(index))
    }
}
