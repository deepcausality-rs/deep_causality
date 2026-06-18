/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Maximum-cardinality search (MCS) over the internal undirected graph.
//!
//! Ported verbatim from the authoritative `cliquepicking_rs::chordal`. For a
//! chordal graph the MCS ordering is a perfect elimination ordering, which the
//! clique-tree construction relies on. Chordality of the input is *assumed*, not
//! checked (mirroring the reference and the repo's `brcd_mec` precondition).

use crate::dag_sampling::graph::Graph;

/// Returns a maximum-cardinality-search ordering of the graph's vertices.
///
/// At each step it picks an unvisited vertex with the most already-visited
/// neighbors. For a chordal graph the returned ordering is a perfect elimination
/// ordering.
pub(crate) fn mcs(g: &Graph) -> Vec<usize> {
    let mut ordering = Vec::new();
    let mut sets: Vec<Vec<usize>> = vec![Vec::new(); g.n];
    let mut cardinality = vec![0usize; g.n];
    let mut max_cardinality = 0usize;

    sets[0] = (0..g.n).collect();

    let mut idx = 0;
    while idx < g.n {
        while max_cardinality > 0 && sets[max_cardinality].is_empty() {
            max_cardinality -= 1;
        }
        let u = sets[max_cardinality].pop().unwrap();
        if cardinality[u] == usize::MAX {
            continue;
        }
        idx += 1;
        ordering.push(u);
        cardinality[u] = usize::MAX;
        for &v in g.neighbors(u) {
            if cardinality[v] < g.n {
                cardinality[v] += 1;
                sets[cardinality[v]].push(v);
            }
        }
        max_cardinality += 1;
    }
    ordering
}
