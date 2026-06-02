/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! F-node augmentation and cut-configuration enumeration.
//!
//! BRCD scores a root-cause candidate by augmenting the causal graph with an
//! intervention indicator **F-node** (`F = 0` on the normal rows, `F = 1` on the
//! anomalous rows) pointing at the candidate, then integrating over the DAGs the
//! candidate's local orientations allow. Two pieces live here, ported from the
//! authoritative `ctx/next/brcd/brcd.py`:
//!
//! * [`f_node_indicator`] — the `0/1` F-node column for the concatenated
//!   normal+anomalous frame.
//! * [`get_configurations_multi`] — `getConfigurations_multi` (L1213): enumerate
//!   every orientation of the undirected edges **incident on the candidate set**,
//!   keeping those that complete (under Meek) to a valid I-CPDAG — acyclic and
//!   introducing no new unshielded collider at any candidate. The returned
//!   configurations are already Meek-completed (the reference completes them in
//!   `sampleAugmentedGraphs`; we fold that in).
//! * [`augmented_graph`] — adds the `FNODE` vertex with `FNODE → candidate` arcs
//!   to a completed configuration, yielding the structural graph whose Markov
//!   equivalence class the next stage sizes and samples.

use crate::brcd::brcd_error::{BrcdError, BrcdErrorEnum};
use crate::brcd::brcd_validity::{baseline_parents, is_valid_configuration};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;
use std::collections::BTreeSet;

/// Upper bound on the number of incident undirected edges enumerated: the
/// configuration space is `2^edges`, refused beyond this.
pub const MAX_CONFIG_EDGES: usize = 16;

/// Builds the F-node indicator column for the concatenated frame: `false` for
/// each of the `n_normal` normal rows, then `true` for each of the `n_anomalous`
/// anomalous rows.
pub fn f_node_indicator(n_normal: usize, n_anomalous: usize) -> Vec<bool> {
    let mut f = vec![false; n_normal];
    f.extend(std::iter::repeat_n(true, n_anomalous));
    f
}

/// Enumerates the valid cut configurations for the candidate set `targets`:
/// every orientation of the undirected edges incident on any target, Meek-
/// completed, kept iff the completion is acyclic and adds no new unshielded
/// collider at any target. Mirrors `getConfigurations_multi`.
///
/// The returned graphs are Meek-completed clones of `cpdag`.
///
/// # Errors
/// * [`BrcdErrorEnum::NodeOutOfBounds`] if a target is not a vertex of `cpdag`.
/// * [`BrcdErrorEnum::ConfigSpaceTooLarge`] if more than [`MAX_CONFIG_EDGES`]
///   undirected edges are incident on the candidate set.
pub fn get_configurations_multi<N: Clone>(
    cpdag: &MixedGraph<N>,
    targets: &[usize],
) -> Result<Vec<MixedGraph<N>>, BrcdError> {
    let n = cpdag.num_vertices();
    if targets.iter().any(|&t| t >= n) {
        return Err(BrcdError(BrcdErrorEnum::NodeOutOfBounds));
    }

    let incident = incident_undirected_edges(cpdag, targets);
    let e = incident.len();
    if e > MAX_CONFIG_EDGES {
        return Err(BrcdError(BrcdErrorEnum::ConfigSpaceTooLarge { edges: e }));
    }

    // Baseline parents at each target, captured before any orientation, so a
    // "new" collider is judged against the original CPDAG.
    let baseline = baseline_parents(cpdag, targets);

    let mut configs = Vec::new();
    // 2^e orientations: bit i chooses the direction of incident edge i.
    for combo in 0..(1usize << e) {
        let mut g = cpdag.clone();
        for (i, &(a, b)) in incident.iter().enumerate() {
            if (combo >> i) & 1 == 0 {
                g.orient(a, b)
                    .expect("incident edge is undirected in the clone");
            } else {
                g.orient(b, a)
                    .expect("incident edge is undirected in the clone");
            }
        }
        // Meek-completes `g` in place and validates it.
        if is_valid_configuration(&mut g, targets, &baseline) {
            configs.push(g);
        }
    }
    Ok(configs)
}

/// Returns the structural F-node-augmented graph for one completed configuration:
/// a copy of `config` (its arcs and any remaining undirected edges) with a new
/// `FNODE` vertex (index `config.num_vertices()`) and a directed arc
/// `FNODE → root` for every `root` in `roots`. The payload is dropped (the
/// augmented graph is used only for Markov-equivalence-class sizing/sampling).
///
/// # Errors
/// [`BrcdErrorEnum::NodeOutOfBounds`] if a root is not a vertex of `config`.
pub fn augmented_graph<N>(
    config: &MixedGraph<N>,
    roots: &[usize],
) -> Result<MixedGraph<()>, BrcdError> {
    let n = config.num_vertices();
    if roots.iter().any(|&r| r >= n) {
        return Err(BrcdError(BrcdErrorEnum::NodeOutOfBounds));
    }
    let fnode = n;
    let data = CausalTensor::new(vec![(); n + 1], vec![n + 1])
        .expect("unit payload of length n+1 is a valid 1-D tensor");
    let mut aug = MixedGraph::<()>::new(n + 1, data, 0)
        .expect("n+1 ≥ 1 with a matching payload and cursor 0");

    // Copy every edge with its original endpoint marks.
    for (&(a, b), edge) in config.edges() {
        aug.add_edge(a, b, edge.lo, edge.hi)
            .expect("copying a canonical edge into a fresh graph cannot conflict");
    }
    // FNODE → each root.
    for &root in roots {
        aug.add_arc(fnode, root)
            .expect("FNODE → root is a fresh edge to a new vertex");
    }
    Ok(aug)
}

/// Collects the undirected edges incident on any node in `targets`, as canonical
/// `(min, max)` pairs in ascending order (deterministic).
fn incident_undirected_edges<N>(cpdag: &MixedGraph<N>, targets: &[usize]) -> Vec<(usize, usize)> {
    let target_set: BTreeSet<usize> = targets.iter().copied().collect();
    cpdag
        .undirected_edges()
        .into_iter()
        .filter(|&(a, b)| target_set.contains(&a) || target_set.contains(&b))
        .collect()
}
