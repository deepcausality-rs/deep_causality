/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// A node test and an edge test that select the part of a graph an algorithm runs on.
///
/// A node passes when it is live and `node` holds for its payload; an edge passes when `edge`
/// holds for its payload and both of its ends pass. A pair of closures is a projection, and a
/// caller's own type, an enum with one variant per scope for example, can implement the trait so
/// that every test is dispatched statically.
pub trait Projection<N, W> {
    /// Whether a node with this payload is in the projection.
    fn node(&self, node: &N) -> bool;
    /// Whether an edge with this payload is in the projection.
    fn edge(&self, edge: &W) -> bool;
}

impl<N, W, F, G> Projection<N, W> for (F, G)
where
    F: Fn(&N) -> bool,
    G: Fn(&W) -> bool,
{
    fn node(&self, node: &N) -> bool {
        (self.0)(node)
    }

    fn edge(&self, edge: &W) -> bool {
        (self.1)(edge)
    }
}
