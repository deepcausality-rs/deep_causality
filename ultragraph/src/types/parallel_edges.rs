/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// How an algorithm counts the parallel edges between an ordered pair of nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ParallelEdges {
    /// The edges between an ordered pair count as one connection; a weighted path takes the
    /// lightest of them. Undirected algorithms see one connection per unordered pair.
    Merge,
    /// Every edge counts as its own link. Undirected algorithms see `max(forward, reverse)`
    /// edges per unordered pair.
    Distinct,
}
