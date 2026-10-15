/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared undirected-view adjacency used by the three biconnectivity algorithms
//! (`articulation_points`, `bridges`, `biconnected_components`).
//!
//! The three algorithms are defined on undirected graphs over directed adjacency. This helper
//! materializes a symmetric CSR view: every undirected edge `{u, v}` (with `u != v`) is
//! represented as two half-edges that share a stable `edge_id`. The shared id is what lets the
//! DFS distinguish "the edge we arrived on" from a real back edge.
//!
//! Self-loops are discarded. How many undirected edges an unordered pair `{u, v}` gets is the
//! `ParallelEdges` mode's decision: one under `Merge`, and `max(forward, reverse)` under
//! `Distinct`, where a reverse-orientation edge pairs with a forward one to make a single
//! undirected edge and any unpaired edges are parallel edges of their own. The count changes
//! the bridges (two parallel edges between `u` and `v` are not a bridge under `Distinct`, one
//! connection is under `Merge`); the articulation points and the biconnected components, which
//! are vertex sets, are the same under both.

use crate::ParallelEdges;
use crate::types::storage::graph_csm::adjacency::Adjacency;
use alloc::collections::BTreeMap;
use alloc::vec;
use alloc::vec::Vec;

/// Symmetric CSR adjacency over the undirected view of an adjacency.
///
/// - `offsets` has length `V + 1`, where `V` is the slot count; `offsets[v]..offsets[v+1]`
///   slices the half-edges incident to vertex `v`, and a tombstone's range is empty.
/// - `targets[i]` is the neighbor reached by half-edge `i`.
/// - `edge_ids[i]` is the canonical id `0..num_edges` of the undirected
///   edge containing half-edge `i`; the two paired half-edges share the id.
///
/// The count of distinct undirected edges is `targets.len() / 2`.
pub(crate) struct SymmetricAdjacency {
    pub(crate) offsets: Vec<usize>,
    pub(crate) targets: Vec<usize>,
    pub(crate) edge_ids: Vec<usize>,
}

pub(crate) fn build_symmetric_adjacency<W, A: Adjacency<W>>(g: &A) -> SymmetricAdjacency {
    let n = g.slots();

    // Count occurrences of each ordered direction for every canonical pair (a, b) with a < b.
    let mut counts: BTreeMap<(usize, usize), (usize, usize)> = BTreeMap::new();
    for u in 0..n {
        for (v, _) in g.out_edges(u) {
            if u == v {
                continue;
            }
            let (a, b, is_forward) = if u < v { (u, v, true) } else { (v, u, false) };
            let entry = counts.entry((a, b)).or_insert((0usize, 0usize));
            if is_forward {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }
    }

    // Expand each canonical pair into the number of undirected edges the mode gives it. Each
    // occurrence below receives its own `edge_id`, which is what lets the Tarjan DFS
    // distinguish "the edge we arrived on" from a parallel sibling edge.
    let mut canonical: Vec<(usize, usize)> = Vec::new();
    for (&(a, b), &(f, r)) in counts.iter() {
        let m = match g.mode() {
            ParallelEdges::Merge => 1,
            ParallelEdges::Distinct => f.max(r),
        };
        for _ in 0..m {
            canonical.push((a, b));
        }
    }
    // `counts` is a BTreeMap, so the loop above already pushed `canonical` in
    // ascending (a, b) order with runs of parallel edges contiguous. Edge-id
    // assignment is deterministic without an explicit sort.

    let mut degree: Vec<usize> = vec![0; n];
    for &(a, b) in &canonical {
        degree[a] += 1;
        degree[b] += 1;
    }

    let mut offsets: Vec<usize> = vec![0; n + 1];
    for i in 0..n {
        offsets[i + 1] = offsets[i] + degree[i];
    }
    let total = offsets[n];

    let mut targets: Vec<usize> = vec![0; total];
    let mut edge_ids: Vec<usize> = vec![0; total];
    let mut cursor: Vec<usize> = offsets[..n].to_vec();

    for (id, &(a, b)) in canonical.iter().enumerate() {
        let pa = cursor[a];
        cursor[a] += 1;
        targets[pa] = b;
        edge_ids[pa] = id;

        let pb = cursor[b];
        cursor[b] += 1;
        targets[pb] = a;
        edge_ids[pb] = id;
    }

    SymmetricAdjacency {
        offsets,
        targets,
        edge_ids,
    }
}

// Coverage of this helper is delivered by the integration tests of the
// three biconnectivity algorithms (see ultragraph/tests/...). All branches
// of `build_symmetric_adjacency` are exercised by their fixtures:
// empty graph, isolated vertices, self-loops, single directed edge,
// both directions stored, multi-edges, mixed.
