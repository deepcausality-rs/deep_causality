/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::validity::{
    baseline_parents, has_new_unshielded_collider_any, has_new_unshielded_collider_at,
    is_valid_configuration,
};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;
use std::collections::{BTreeMap, BTreeSet};

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

fn parents_set(items: &[usize]) -> BTreeSet<usize> {
    items.iter().copied().collect()
}

#[test]
fn new_unshielded_collider_is_flagged() {
    // 0 → 2 ← 1, with 0 and 1 non-adjacent: unshielded collider at 2.
    let mut g = graph(3);
    g.add_arc(0, 2).unwrap();
    g.add_arc(1, 2).unwrap();
    // Baseline had no parents at node 2 ⇒ this collider is new.
    assert!(has_new_unshielded_collider_at(&g, 2, &parents_set(&[])));
}

#[test]
fn preexisting_collider_is_not_flagged() {
    // Same collider, but both parents were already in the baseline ⇒ not new.
    let mut g = graph(3);
    g.add_arc(0, 2).unwrap();
    g.add_arc(1, 2).unwrap();
    assert!(!has_new_unshielded_collider_at(
        &g,
        2,
        &parents_set(&[0, 1])
    ));
}

#[test]
fn shielded_collider_is_not_a_collider() {
    // 0 → 2 ← 1 but 0 and 1 ARE adjacent (0 — 1): shielded, not an unshielded collider.
    let mut g = graph(3);
    g.add_arc(0, 2).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_undirected(0, 1).unwrap();
    assert!(!has_new_unshielded_collider_at(&g, 2, &parents_set(&[])));
}

#[test]
fn one_baseline_parent_still_counts_as_new() {
    // New collider needs BOTH parents already in baseline to be "not new".
    let mut g = graph(3);
    g.add_arc(0, 2).unwrap();
    g.add_arc(1, 2).unwrap();
    assert!(has_new_unshielded_collider_at(&g, 2, &parents_set(&[0])));
}

#[test]
fn any_target_with_new_collider_is_flagged() {
    let mut g = graph(4);
    g.add_arc(0, 2).unwrap();
    g.add_arc(1, 2).unwrap(); // collider at 2
    let base = baseline_parents(&g, &[3]); // unrelated target
    assert!(!has_new_unshielded_collider_any(&g, &[3], &base));
    let base2 = baseline_parents(&graph(4), &[2]); // baseline (empty) at 2
    assert!(has_new_unshielded_collider_any(&g, &[2], &base2));
}

#[test]
fn target_absent_from_baseline_map_treats_baseline_as_empty() {
    // 0 → 2 ← 1 (new collider at 2), but the baseline map has no entry for 2.
    // The missing entry must be treated as an empty baseline parent set, so the
    // collider counts as new. Exercises the `unwrap_or(&empty)` fallback.
    let mut g = graph(3);
    g.add_arc(0, 2).unwrap();
    g.add_arc(1, 2).unwrap();
    let empty_map = BTreeMap::new();
    assert!(has_new_unshielded_collider_any(&g, &[2], &empty_map));
}

#[test]
fn valid_configuration_accepts_clean_completion() {
    // 0 → 1, 1 — 2 completes to 0 → 1 → 2: acyclic, no new collider at the target.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    let base = baseline_parents(&g, &[2]);
    assert!(is_valid_configuration(&mut g, &[2], &base));
}

#[test]
fn valid_configuration_rejects_cycle() {
    // A directed cycle 0 → 1 → 2 → 0 is not a valid configuration.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(2, 0).unwrap();
    let base = baseline_parents(&g, &[0, 1, 2]);
    assert!(!is_valid_configuration(&mut g, &[0, 1, 2], &base));
}

#[test]
fn valid_configuration_rejects_new_collider() {
    // 0 → 2, 1 → 2 with 0,1 non-adjacent: a new unshielded collider at 2.
    let mut g = graph(3);
    g.add_arc(0, 2).unwrap();
    g.add_arc(1, 2).unwrap();
    let empty = baseline_parents(&graph(3), &[2]); // baseline has no parents at 2
    assert!(!is_valid_configuration(&mut g, &[2], &empty));
}
