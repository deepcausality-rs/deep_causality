/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EdgeKey;
use alloc::vec::Vec;

pub trait GraphView<N, W>
where
    W: EdgeKey,
{
    // State Inspection
    fn is_frozen(&self) -> bool;

    fn is_empty(&self) -> bool;

    // Node Inspection
    fn contains_node(&self, index: usize) -> bool;
    fn get_node(&self, index: usize) -> Option<&N>;
    fn number_nodes(&self) -> usize;

    // Edge Inspection
    /// True when an edge from `a` to `b` exists with any key and both ends are live.
    fn contains_edge(&self, a: usize, b: usize) -> bool;
    /// True when the edge from `a` to `b` with key `key` exists and both ends are live.
    fn contains_edge_keyed(&self, a: usize, b: usize, key: W::Key) -> bool;
    fn number_edges(&self) -> usize;

    fn get_all_nodes(&self) -> Vec<&N>;
    fn get_edges(&self, source: usize) -> Option<Vec<(usize, &W)>>;

    fn get_last_index(&self) -> Option<usize>;

    // Root Node Inspection
    fn contains_root_node(&self) -> bool;
    fn get_root_node(&self) -> Option<&N>;
    fn get_root_index(&self) -> Option<usize>;
}
