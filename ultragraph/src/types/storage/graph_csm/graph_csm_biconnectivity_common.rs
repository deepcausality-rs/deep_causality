/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Shared undirected-view adjacency used by the three biconnectivity algorithms
//! (`articulation_points`, `bridges`, `biconnected_components`).
//!
//! The three algorithms are defined on undirected graphs. `CsmGraph` stores
//! directed adjacency. This helper materializes a symmetric CSR view: every
//! distinct undirected edge `{u, v}` (with `u != v`) is represented as two
//! half-edges that share a stable `edge_id`. The shared id is what lets the
//! DFS distinguish "the edge we arrived on" from a real back edge — robust
//! to multi-edges in the input.
//!
//! Self-loops and duplicate edge orientations are discarded.

use crate::{CsmGraph, GraphView};
use std::collections::HashSet;

/// Symmetric CSR adjacency over the undirected view of a `CsmGraph`.
///
/// - `offsets` has length `V + 1`; `offsets[v]..offsets[v+1]` slices the
///   half-edges incident to vertex `v`.
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

pub(crate) fn build_symmetric_adjacency<N, W>(g: &CsmGraph<N, W>) -> SymmetricAdjacency
where
    N: Clone,
    W: Clone + Default,
{
    let n = g.number_nodes();

    // Collect distinct canonical undirected edges (a, b) with a < b.
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    let mut canonical: Vec<(usize, usize)> = Vec::new();
    for u in 0..n {
        let start = g.forward_edges.offsets[u];
        let end = g.forward_edges.offsets[u + 1];
        for &v in &g.forward_edges.targets[start..end] {
            if u == v {
                continue;
            }
            let (a, b) = if u < v { (u, v) } else { (v, u) };
            if seen.insert((a, b)) {
                canonical.push((a, b));
            }
        }
    }
    // Sort to make edge_id assignment deterministic regardless of CSR layout.
    canonical.sort_unstable();

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
