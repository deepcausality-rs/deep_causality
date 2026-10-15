/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::storage::graph_csm::adjacency::Adjacency;
use crate::{CentralityGraphAlgorithms, CsmGraph, EdgeKey, GraphError, ParallelEdges};
use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;

impl<N, W> CentralityGraphAlgorithms<N, W> for CsmGraph<N, W>
where
    N: Clone,
    W: Clone + Default + EdgeKey,
{
    /// Calculates the betweenness centrality of each node in the graph.
    fn betweenness_centrality(
        &self,
        directed: bool,
        normalized: bool,
    ) -> Result<Vec<(usize, f64)>, GraphError> {
        betweenness_centrality_impl(self, directed, normalized)
    }

    /// Calculates betweenness centrality across a specific set of critical pathways.
    fn pathway_betweenness_centrality(
        &self,
        pathways: &[(usize, usize)],
        directed: bool,
        normalized: bool,
    ) -> Result<Vec<(usize, f64)>, GraphError> {
        pathway_betweenness_centrality_impl(self, pathways, directed, normalized)
    }
}

/// Brandes' algorithm over the passing nodes. Parallel edges enter the path counts, so the
/// mode matters here: under `Merge` the edges between a pair are one connection, under
/// `Distinct` each is a path of its own (directed), or `max(forward, reverse)` of them are
/// (undirected).
pub(crate) fn betweenness_centrality_impl<W, A: Adjacency<W>>(
    a: &A,
    directed: bool,
    normalized: bool,
) -> Result<Vec<(usize, f64)>, GraphError> {
    // Arrays are indexed by position and sized by the slot count; the node count that enters
    // the result (the small-graph case and the normalisation) is the passing count.
    let live = a.passing_count();
    if live == 0 {
        return Ok(Vec::new());
    }
    let slots = a.slots();

    let mut centrality = vec![0.0; slots];

    // Betweenness centrality is 0 for graphs with less than 3 nodes.
    if live < 3 {
        return Ok(live_scores(a, centrality));
    }

    for s in 0..slots {
        if !a.passes(s) {
            continue;
        }
        let (_, sigma, pred, mut stack) = brandes_bfs_and_path_counting(a, s, directed)?;

        // Accumulation
        let mut delta = vec![0.0; slots];
        while let Some(w) = stack.pop() {
            for &v in &pred[w] {
                let sigma_v = sigma[v];
                let sigma_w = sigma[w];
                delta[v] += (sigma_v / sigma_w) * (1.0 + delta[w]);
            }
            if w != s {
                centrality[w] += delta[w];
            }
        }
    }

    // It aligns the code with the standard formal definition.
    if !directed {
        for score in centrality.iter_mut() {
            *score /= 2.0;
        }
    }

    // Normalization
    if normalized {
        // At least three live nodes reach this point, so the scale is at least 2.
        let scale = if directed {
            ((live - 1) * (live - 2)) as f64
        } else {
            ((live - 1) * (live - 2)) as f64 / 2.0
        };
        for score in centrality.iter_mut() {
            *score /= scale;
        }
    }

    Ok(live_scores(a, centrality))
}

pub(crate) fn pathway_betweenness_centrality_impl<W, A: Adjacency<W>>(
    a: &A,
    pathways: &[(usize, usize)],
    directed: bool,
    normalized: bool,
) -> Result<Vec<(usize, f64)>, GraphError> {
    if a.passing_count() == 0 {
        return Ok(vec![]);
    }
    let num_nodes = a.slots();

    let mut centrality = vec![0.0; num_nodes];
    let mut valid_pathways = 0;

    // Unlike the global version, we cannot easily group by source,
    // as the dependency calculation is specific to each target `t`.
    for &(s, t) in pathways {
        if !a.passes(s) || !a.passes(t) {
            continue;
        }

        if s == t {
            valid_pathways += 1;
            continue;
        }

        // We need a fresh BFS for each s->t pair to get the correct stack order
        let (dist, sigma, pred, mut stack) = brandes_bfs_and_path_counting(a, s, directed)?;

        // If t is not reachable from s, this pathway contributes nothing.
        if dist[t].is_none() {
            continue;
        }
        valid_pathways += 1;

        let mut delta = vec![0.0; num_nodes];

        // Process nodes in reverse order from the stack
        while let Some(w) = stack.pop() {
            // The contribution to dependency is only propagated from nodes on a shortest path to t.
            // This can be simplified: if a node `w` is part of an s-t path, its `delta` will be non-zero
            // if one of its successors on the shortest path has non-zero delta. We start this at `t`.
            if w == t {
                // This is the starting point of our back-propagation for this s-t path
                delta[w] = 1.0;
            }

            // Every node on the stack was reached, so `sigma[w]` is at least 1.
            for &v in &pred[w] {
                // The dependency of v is its share of the dependency of w.
                delta[v] += (sigma[v] / sigma[w]) * delta[w];
            }
        }

        // Add the accumulated dependencies to the final centrality scores
        // The endpoints s and t themselves do not get centrality credit from this path.
        for i in 0..num_nodes {
            if i != s && i != t {
                centrality[i] += delta[i];
            }
        }
    }

    if normalized && valid_pathways > 0 {
        let scale = valid_pathways as f64;
        for score in &mut centrality {
            *score /= scale;
        }
    }

    Ok(live_scores(a, centrality))
}

/// Pairs each passing position with its score; the others are left out.
fn live_scores<W, A: Adjacency<W>>(a: &A, scores: Vec<f64>) -> Vec<(usize, f64)> {
    scores
        .into_iter()
        .enumerate()
        .filter(|(position, _)| a.passes(*position))
        .collect()
}

/// The neighbours of a node in the undirected view, with the multiplicity the mode gives them.
///
/// `out` and `inn` are the targets of the node's out-edges and the sources of its in-edges,
/// each sorted ascending. Under `Merge` every distinct neighbour appears once; under `Distinct`
/// a neighbour `v` appears `max(f, r)` times, `f` being the edges to `v` and `r` the edges from
/// `v`, the rule the biconnectivity algorithms count undirected edges by.
fn undirected_neighbours(out: &[usize], inn: &[usize], mode: ParallelEdges, into: &mut Vec<usize>) {
    let (mut i, mut j) = (0, 0);
    loop {
        let next = match (out.get(i), inn.get(j)) {
            (Some(&o), Some(&n)) => o.min(n),
            (Some(&o), None) => o,
            (None, Some(&n)) => n,
            (None, None) => return,
        };
        let f = run_length(out, &mut i, next);
        let r = run_length(inn, &mut j, next);
        let multiplicity = match mode {
            ParallelEdges::Merge => 1,
            ParallelEdges::Distinct => f.max(r),
        };
        into.extend(core::iter::repeat_n(next, multiplicity));
    }
}

/// Advances `i` past the run of `value` in `xs` and returns the run's length.
fn run_length(xs: &[usize], i: &mut usize, value: usize) -> usize {
    let start = *i;
    while *i < xs.len() && xs[*i] == value {
        *i += 1;
    }
    *i - start
}

/// Brandes' BFS and path-counting phase from `s`. Returns (dist, sigma, pred, stack).
#[allow(clippy::type_complexity)]
#[inline]
fn brandes_bfs_and_path_counting<W, A: Adjacency<W>>(
    a: &A,
    s: usize,
    directed: bool,
) -> Result<(Vec<Option<usize>>, Vec<f64>, Vec<Vec<usize>>, Vec<usize>), GraphError> {
    let num_nodes = a.slots();
    let mode = a.mode();
    let mut stack = Vec::new(); // S in Brandes' algorithm
    let mut queue = VecDeque::new();
    let mut dist = vec![None; num_nodes];
    let mut sigma = vec![0.0; num_nodes]; // Number of shortest paths
    let mut pred: Vec<Vec<usize>> = vec![Vec::new(); num_nodes]; // Predecessors on shortest paths
    let mut neighbors_to_process = Vec::new(); // Reused to avoid reallocations
    let mut out = Vec::new();
    let mut inn = Vec::new();

    dist[s] = Some(0);
    sigma[s] = 1.0;
    queue.push_back(s);

    while let Some(v) = queue.pop_front() {
        stack.push(v);
        neighbors_to_process.clear(); // Clear for current node
        if directed {
            // The adjacency is sorted by target, so under `Merge` the parallel edges to one
            // target are adjacent and `dedup` leaves one connection.
            neighbors_to_process.extend(a.out_edges(v).map(|(target, _)| target));
            if mode == ParallelEdges::Merge {
                neighbors_to_process.dedup();
            }
        } else {
            // For undirected, consider both forward and backward neighbors, with the
            // multiplicity the mode gives them. Both lists are sorted, so the order is
            // deterministic.
            out.clear();
            out.extend(a.out_edges(v).map(|(target, _)| target));
            inn.clear();
            inn.extend(a.in_edges(v).map(|(source, _)| source));
            undirected_neighbours(&out, &inn, mode, &mut neighbors_to_process);
        }

        for &w in &neighbors_to_process {
            let v_dist = dist[v].ok_or(GraphError::AlgorithmError("Distance for v not set"))?;
            // Path discovery
            if dist[w].is_none() {
                dist[w] = Some(v_dist + 1);
                queue.push_back(w);
            }
            // Path counting
            if dist[w].ok_or(GraphError::AlgorithmError("Distance for w not set"))? == v_dist + 1 {
                sigma[w] += sigma[v];
                pred[w].push(v);
            }
        }
    }
    Ok((dist, sigma, pred, stack))
}
