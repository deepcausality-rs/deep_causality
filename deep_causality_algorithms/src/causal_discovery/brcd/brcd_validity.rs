/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Validity check for a candidate orientation: it must not introduce a *new*
//! unshielded collider, and its directed-arc projection must stay acyclic.
//!
//! A triple `a → t ← b` with `a` and `b` non-adjacent is an **unshielded
//! collider** at `t`. When BRCD orients a cut configuration and completes it
//! under Meek, a configuration is only a legal I-CPDAG if it creates no
//! unshielded collider at a target node that was not already present in the
//! baseline CPDAG (and stays acyclic). This mirrors the reference BRCD checks
//! `has_new_unshielded_collider_at` / `_is_valid_configuration_multi`.

use crate::brcd::brcd_meek::meek_complete;
use deep_causality_topology::MixedGraph;
use std::collections::{BTreeMap, BTreeSet};

/// Returns `true` if `node` has a **new** unshielded collider `a → node ← b`
/// (with `a`, `b` non-adjacent) that is **not** already accounted for by
/// `baseline_parents` — the node's parent set in the baseline CPDAG.
///
/// A collider is "not new" only when *both* of its parents were already
/// baseline parents of the node.
pub fn has_new_unshielded_collider_at<N>(
    graph: &MixedGraph<N>,
    node: usize,
    baseline_parents: &BTreeSet<usize>,
) -> bool {
    let parents = graph.parents(node);
    for i in 0..parents.len() {
        for j in (i + 1)..parents.len() {
            let (a, b) = (parents[i], parents[j]);
            // Unshielded (a, b non-adjacent) and not already a baseline collider
            // (both parents present in the baseline).
            let shielded = graph.is_adjacent(a, b);
            let in_baseline = baseline_parents.contains(&a) && baseline_parents.contains(&b);
            if !(shielded || in_baseline) {
                return true;
            }
        }
    }
    false
}

/// Returns `true` if **any** node in `targets` gained a new unshielded collider,
/// each checked against its baseline parent set in `baseline`.
pub fn has_new_unshielded_collider_any<N>(
    graph: &MixedGraph<N>,
    targets: &[usize],
    baseline: &BTreeMap<usize, BTreeSet<usize>>,
) -> bool {
    let empty = BTreeSet::new();
    targets.iter().any(|&node| {
        has_new_unshielded_collider_at(graph, node, baseline.get(&node).unwrap_or(&empty))
    })
}

/// Completes a candidate orientation under Meek (in place), then validates it:
/// the arc projection must be acyclic **and** introduce no new unshielded
/// collider at any node in `targets`. Returns `true` if the configuration is a
/// legal I-CPDAG.
///
/// `baseline` maps each target node to its parent set in the baseline CPDAG,
/// captured before the candidate orientation was applied.
pub fn is_valid_configuration<N>(
    graph: &mut MixedGraph<N>,
    targets: &[usize],
    baseline: &BTreeMap<usize, BTreeSet<usize>>,
) -> bool {
    meek_complete(graph);
    if graph.has_cycle() {
        return false;
    }
    !has_new_unshielded_collider_any(graph, targets, baseline)
}

/// Collects the baseline parent set of each target node from `graph`, for use as
/// the `baseline` argument before a candidate orientation is applied.
pub fn baseline_parents<N>(
    graph: &MixedGraph<N>,
    targets: &[usize],
) -> BTreeMap<usize, BTreeSet<usize>> {
    targets
        .iter()
        .map(|&node| (node, graph.parents(node).into_iter().collect()))
        .collect()
}
