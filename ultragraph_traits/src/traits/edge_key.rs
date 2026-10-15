/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// The part of an edge payload that identifies the edge.
///
/// A graph holds at most one edge per source, target and key. Adding an edge whose key an
/// existing edge between the same pair already carries replaces that edge's payload; a payload
/// with another key is a second edge between the pair. The key is `Ord` because a frozen graph
/// sorts each node's edges by target, then key.
///
/// A payload that is a weight, or nothing, returns `()`: the unit and primitive numeric types
/// implement the trait that way, so such a graph holds at most one edge per ordered pair.
pub trait EdgeKey {
    /// What distinguishes two edges between one pair of nodes.
    type Key: Copy + Ord;

    /// The key of this payload.
    fn edge_key(&self) -> Self::Key;
}
