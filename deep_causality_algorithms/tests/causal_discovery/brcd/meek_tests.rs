/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::brcd_meek::meek_complete;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{EdgeKind, MixedGraph};

/// A structural `MixedGraph` over `n` nodes (unit payload).
fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

#[test]
fn r1_orients_to_avoid_new_collider() {
    // 0 → 1, 1 — 2, with 0 and 2 non-adjacent ⇒ R1 forces 1 → 2.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    meek_complete(&mut g);
    assert_eq!(g.edge_kind(1, 2), Some(EdgeKind::Directed));
    assert_eq!(g.parents(2), vec![1]);
}

#[test]
fn r2_orients_to_avoid_cycle() {
    // 0 → 1 → 2, 0 — 2 ⇒ R2 forces 0 → 2 (else 0→1→2→0 cycle).
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_undirected(0, 2).unwrap();
    meek_complete(&mut g);
    assert_eq!(g.edge_kind(0, 2), Some(EdgeKind::Directed));
    assert!(g.parents(2).contains(&0));
}

#[test]
fn r3_orients_common_child() {
    // 2 → 1, 3 → 1, 0 — 1, 0 — 2, 0 — 3, with 2 and 3 non-adjacent ⇒ R3 forces 0 → 1.
    let mut g = graph(4);
    g.add_arc(2, 1).unwrap();
    g.add_arc(3, 1).unwrap();
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(0, 2).unwrap();
    g.add_undirected(0, 3).unwrap();
    meek_complete(&mut g);
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Directed));
    assert!(g.parents(1).contains(&0));
}

#[test]
fn orientation_propagates_across_passes() {
    // 0 → 1, 1 — 2, 2 — 3; 0⊥2 and 1⊥3.
    // Pass 1: R1 orients 1 → 2 (0→1, 1—2, 0⊥2).
    // Pass 2: R1 orients 2 → 3 (1→2, 2—3, 1⊥3).
    let mut g = graph(4);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.add_undirected(2, 3).unwrap();
    meek_complete(&mut g);
    assert_eq!(g.edge_kind(1, 2), Some(EdgeKind::Directed));
    assert_eq!(g.edge_kind(2, 3), Some(EdgeKind::Directed));
    assert_eq!(g.parents(3), vec![2]);
}

#[test]
fn fully_directed_dag_is_unchanged() {
    // Arcs-only acyclic input: completion is a no-op.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(0, 2).unwrap();
    let before = g.edges().clone();
    meek_complete(&mut g);
    assert_eq!(g.edges(), &before);
}

#[test]
fn unforced_undirected_edge_stays_undirected() {
    // A lone undirected edge with no compelling structure remains undirected.
    let mut g = graph(2);
    g.add_undirected(0, 1).unwrap();
    meek_complete(&mut g);
    assert_eq!(g.edge_kind(0, 1), Some(EdgeKind::Undirected));
}

#[test]
fn completion_is_idempotent() {
    let mut g = graph(4);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.add_undirected(2, 3).unwrap();
    meek_complete(&mut g);
    let once = g.edges().clone();
    meek_complete(&mut g);
    assert_eq!(g.edges(), &once);
}
