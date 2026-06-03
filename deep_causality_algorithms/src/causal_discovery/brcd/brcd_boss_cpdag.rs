/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! DAG → CPDAG conversion for the learned BOSS order.
//!
//! Turns the per-variable parent sets produced by the order search into the
//! CPDAG (essential graph) BRCD consumes. The construction reuses the existing
//! Meek machinery rather than porting causal-learn's `dag2cpdag` edge-labelling:
//!
//! 1. build the skeleton (undirected) from the DAG;
//! 2. orient every **unshielded collider** `a → c ← b` (with `a`, `b`
//!    non-adjacent) — the only genuinely new pass;
//! 3. close under Meek rules ([`crate::brcd::brcd_meek::meek_complete`]).
//!
//! The result is identical to `dag2cpdag`: edges compelled in every member of
//! the Markov equivalence class stay directed; edges that reverse across the
//! class are left undirected.

use crate::brcd::brcd_meek::meek_complete;
use crate::brcd::{BrcdError, BrcdErrorEnum};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;
use std::collections::BTreeSet;

/// Converts a DAG given as per-variable parent sets into its CPDAG.
///
/// `parents[c]` lists the parents of variable `c`; `parents.len()` is the number
/// of variables. The returned [`MixedGraph`] carries directed arcs for compelled
/// edges and undirected edges for reversible ones, ready to feed `brcd_run`.
///
/// # Errors
/// * [`BrcdErrorEnum::EmptyData`] if there are no variables.
/// * [`BrcdErrorEnum::NodeOutOfBounds`] if a parent index is out of range.
/// * [`BrcdErrorEnum::NotAcyclic`] if the parent sets do not describe a DAG
///   (a self-loop or a directed cycle).
pub fn dag_to_cpdag(parents: &[Vec<usize>]) -> Result<MixedGraph<()>, BrcdError> {
    let n = parents.len();
    if n == 0 {
        return Err(BrcdError(BrcdErrorEnum::EmptyData));
    }
    for (c, ps) in parents.iter().enumerate() {
        for &p in ps {
            if p >= n {
                return Err(BrcdError(BrcdErrorEnum::NodeOutOfBounds));
            }
            if p == c {
                return Err(BrcdError(BrcdErrorEnum::NotAcyclic));
            }
        }
    }
    if !is_dag(parents) {
        return Err(BrcdError(BrcdErrorEnum::NotAcyclic));
    }

    // Skeleton adjacency as canonical (min, max) pairs.
    let mut adjacency: BTreeSet<(usize, usize)> = BTreeSet::new();
    for (c, ps) in parents.iter().enumerate() {
        for &p in ps {
            adjacency.insert((p.min(c), p.max(c)));
        }
    }
    let is_adjacent = |a: usize, b: usize| adjacency.contains(&(a.min(b), a.max(b)));

    // Compelled arcs from unshielded colliders: for each node, every pair of
    // non-adjacent parents `a, b` forces `a → c` and `b → c`.
    let mut compelled: BTreeSet<(usize, usize)> = BTreeSet::new();
    for (c, ps) in parents.iter().enumerate() {
        for i in 0..ps.len() {
            for j in (i + 1)..ps.len() {
                let (a, b) = (ps[i], ps[j]);
                if !is_adjacent(a, b) {
                    compelled.insert((a, c));
                    compelled.insert((b, c));
                }
            }
        }
    }

    // Build the PDAG: v-structure edges directed, the rest undirected.
    let data = CausalTensor::new(vec![(); n], vec![n])
        .map_err(|_| BrcdError(BrcdErrorEnum::DimensionMismatch))?;
    let mut graph = MixedGraph::<()>::new(n, data, 0)
        .map_err(|_| BrcdError(BrcdErrorEnum::DimensionMismatch))?;
    for (c, ps) in parents.iter().enumerate() {
        for &p in ps {
            if compelled.contains(&(p, c)) {
                graph
                    .add_arc(p, c)
                    .map_err(|_| BrcdError(BrcdErrorEnum::DimensionMismatch))?;
            } else {
                graph
                    .add_undirected(p, c)
                    .map_err(|_| BrcdError(BrcdErrorEnum::DimensionMismatch))?;
            }
        }
    }

    // Close under Meek to compel every further-forced edge.
    meek_complete(&mut graph);
    Ok(graph)
}

/// Kahn topological-sort check: `true` iff the parent sets describe a DAG.
fn is_dag(parents: &[Vec<usize>]) -> bool {
    let n = parents.len();
    let mut indegree = vec![0usize; n];
    let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
    for (c, ps) in parents.iter().enumerate() {
        indegree[c] = ps.len();
        for &p in ps {
            children[p].push(c);
        }
    }
    let mut queue: Vec<usize> = (0..n).filter(|&i| indegree[i] == 0).collect();
    let mut processed = 0usize;
    while let Some(v) = queue.pop() {
        processed += 1;
        for &c in &children[v] {
            indegree[c] -= 1;
            if indegree[c] == 0 {
                queue.push(c);
            }
        }
    }
    processed == n
}
