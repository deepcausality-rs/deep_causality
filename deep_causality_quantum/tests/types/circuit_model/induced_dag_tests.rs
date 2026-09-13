/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The induced DAG's queries, on graphs small enough to read by hand.
//!
//! Corner cases: (A) empty graph, (B) one vertex, (C) coinciding start and target in
//! `reaches_avoiding`, (D) an out-of-range edge is ignored, (E) a blocked start, (F) a cycle.

use deep_causality_quantum::InducedDag;
use std::collections::BTreeSet;

fn set(v: &[usize]) -> BTreeSet<usize> {
    v.iter().copied().collect()
}

/// The paper's Example 54 low-level DAG: `X ← Z → Y`, `Z ← W`? No: `c: Z → X`, `d: Z → Y`,
/// `e: W → Z`, so edges `W → Z`, `Z → X`, `Z → Y`. Vertices `X=0, Y=1, Z=2, W=3`.
fn example_54() -> InducedDag {
    let mut g = InducedDag::new(4);
    g.add_edge(3, 2).add_edge(2, 0).add_edge(2, 1);
    g
}

#[test]
fn test_empty_graph_has_no_cycle_and_empty_order() {
    let g = InducedDag::new(0);
    assert!(!g.has_cycle());
    assert_eq!(g.topological_order(), Some(vec![]));
    assert_eq!(g.num_edges(), 0);
}

#[test]
fn test_single_vertex() {
    let g = InducedDag::new(1);
    assert_eq!(g.topological_order(), Some(vec![0]));
    assert!(!g.reaches(0, 0));
    assert!(g.reaches_avoiding(0, &set(&[0]), &set(&[])));
}

#[test]
fn test_out_of_range_edge_is_ignored() {
    let mut g = InducedDag::new(2);
    g.add_edge(0, 5).add_edge(7, 1);
    assert_eq!(g.num_edges(), 0);
}

#[test]
fn test_parents_children_and_reachability_on_example_54() {
    let g = example_54();
    assert_eq!(g.parents(0), vec![2]);
    assert_eq!(g.children(2), vec![0, 1]);
    assert!(g.reaches(3, 0));
    assert!(g.reaches(3, 1));
    assert!(!g.reaches(0, 1));
    assert!(!g.reaches(1, 0));
    assert_eq!(g.topological_order(), Some(vec![3, 2, 0, 1]));
}

#[test]
fn test_alpha_of_example_54_through_blocked_reachability() {
    // Definition 49 with π :: X ↦ {X}, Y ↦ {Y}, W ↦ {W}: Pa(X) = {W} in H, so the blocked set for
    // α(X) is π(W) = {W}. Z reaches X without passing W, so Z ∈ α(X); W is blocked; Y does not
    // reach X. α(X) = {X, Z}, as the paper states on p. 39.
    let g = example_54();
    let target_x = set(&[0]);
    let blocked_w = set(&[3]);
    let alpha_x: Vec<usize> = (0..4)
        .filter(|&v| g.reaches_avoiding(v, &target_x, &blocked_w))
        .collect();
    assert_eq!(alpha_x, vec![0, 2]);
    // With π(W) = {W, Z} blocked, α(X) = {X}.
    let blocked_wz = set(&[2, 3]);
    let alpha_x2: Vec<usize> = (0..4)
        .filter(|&v| g.reaches_avoiding(v, &target_x, &blocked_wz))
        .collect();
    assert_eq!(alpha_x2, vec![0]);
}

#[test]
fn test_blocked_start_never_reaches() {
    let g = example_54();
    assert!(!g.reaches_avoiding(2, &set(&[0]), &set(&[2])));
    assert!(!g.reaches_avoiding(9, &set(&[0]), &set(&[])));
}

#[test]
fn test_cycle_is_detected() {
    let mut g = InducedDag::new(3);
    g.add_edge(0, 1).add_edge(1, 2).add_edge(2, 0);
    assert!(g.has_cycle());
    assert_eq!(g.topological_order(), None);
    assert!(g.reaches(0, 0));
    let mut h = InducedDag::new(1);
    h.add_edge(0, 0);
    assert!(h.has_cycle());
}

#[test]
fn test_edge_bounds_are_exact_at_the_vertex_count() {
    let mut g = InducedDag::new(2);
    g.add_edge(1, 1).add_edge(2, 0).add_edge(0, 2);
    assert_eq!(
        g.edges().collect::<Vec<_>>(),
        vec![(1, 1)],
        "only the in-range self loop stays"
    );
    assert_eq!(g.num_edges(), 1);
    let h = example_54();
    assert_eq!(h.edges().collect::<Vec<_>>(), vec![(2, 0), (2, 1), (3, 2)]);
}

#[test]
fn test_unreachable_target_past_a_visited_vertex() {
    // Every vertex reached from 2 differs from 3, so a search that stops at the first
    // non-target vertex would answer wrongly.
    let g = example_54();
    assert!(!g.reaches(2, 3));
    assert!(!g.reaches(3, 3));
    // A cycle with an unreachable vertex terminates and answers false.
    let mut c = InducedDag::new(4);
    c.add_edge(0, 1).add_edge(1, 2).add_edge(2, 0);
    assert!(!c.reaches(0, 3));
    assert!(c.reaches(1, 1));
}

#[test]
fn test_blocked_target_is_not_reached_through_the_block() {
    let g = example_54();
    // 0 is both the target and blocked: no path may end on a blocked vertex.
    assert!(!g.reaches_avoiding(2, &set(&[0]), &set(&[0])));
    // The same target unblocked is reached.
    assert!(g.reaches_avoiding(2, &set(&[0]), &set(&[])));
}
