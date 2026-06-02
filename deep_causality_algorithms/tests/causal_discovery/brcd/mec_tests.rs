/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::brcd::mec::{
    MEC_ENUM_BOUND, MecError, mec_sample_dag, mec_size, representative_dag,
};
use deep_causality_rand::Xoshiro256;
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::MixedGraph;

fn graph(n: usize) -> MixedGraph<()> {
    let data = CausalTensor::new(vec![(); n], vec![n]).unwrap();
    MixedGraph::new(n, data, 0).unwrap()
}

/// A DAG member is fully directed (every edge an arc) and acyclic.
fn is_fully_directed_dag(g: &MixedGraph<()>) -> bool {
    g.arcs().len() == g.num_edges() && !g.has_cycle()
}

// --- exact MEC size ---------------------------------------------------------

#[test]
fn fully_directed_dag_has_class_size_one() {
    // 0 → 1 → 2, 0 → 2 : already a DAG, equivalence class size 1.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(0, 2).unwrap();
    assert_eq!(mec_size(&g), Ok(1));
}

#[test]
fn empty_graph_has_class_size_one() {
    // No edges: trivially fully directed and acyclic.
    let g = graph(3);
    assert_eq!(mec_size(&g), Ok(1));
}

#[test]
fn single_undirected_edge_has_class_size_two() {
    // 0 — 1 : two orientations (0 → 1, 1 → 0), both valid DAGs.
    let mut g = graph(2);
    g.add_undirected(0, 1).unwrap();
    assert_eq!(mec_size(&g), Ok(2));
}

#[test]
fn undirected_path_of_three_has_class_size_three() {
    // 0 — 1 — 2 (no 0–2 edge): the three moral orientations are
    // 0→1→2, 2→1→0, 0←1→2; the collider 0→1←2 is excluded. Size 3.
    let mut g = graph(3);
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    assert_eq!(mec_size(&g), Ok(3));
}

#[test]
fn undirected_triangle_has_class_size_six() {
    // A complete (chordal) triangle has 3! = 6 acyclic orientations, all moral.
    let mut g = graph(3);
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.add_undirected(0, 2).unwrap();
    assert_eq!(mec_size(&g), Ok(6));
}

#[test]
fn arcs_times_undirected_component_multiplies() {
    // Compelled arc 0 → 1 plus an undirected path 1 — 2 — 3 (class 3).
    // Components orient independently of the arc, so the class size is 1 × 3 = 3.
    let mut g = graph(4);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.add_undirected(2, 3).unwrap();
    assert_eq!(mec_size(&g), Ok(3));
}

#[test]
fn disjoint_components_multiply() {
    // Two disjoint undirected edges: 2 × 2 = 4.
    let mut g = graph(4);
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(2, 3).unwrap();
    assert_eq!(mec_size(&g), Ok(4));
}

// --- representative DAG -----------------------------------------------------

#[test]
fn representative_of_dag_is_the_input() {
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    let rep = representative_dag(&g).unwrap();
    assert_eq!(rep.edges(), g.edges());
    assert_eq!(rep.num_vertices(), g.num_vertices());
}

#[test]
fn representative_of_cpdag_is_a_valid_member() {
    // 0 → 1, 1 — 2 — 3: the representative orients the component into a DAG and
    // keeps the compelled arc.
    let mut g = graph(4);
    g.add_arc(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();
    g.add_undirected(2, 3).unwrap();
    let rep = representative_dag(&g).unwrap();
    assert!(is_fully_directed_dag(&rep));
    // The compelled arc is preserved.
    assert!(rep.arcs().contains(&(0, 1)));
    // No collider at 2 (a moral orientation): at most one parent of 2.
    assert!(rep.parents(2).len() <= 1);
}

// --- uniform sampling -------------------------------------------------------

#[test]
fn sample_is_deterministic_for_a_fixed_seed() {
    let mut g = graph(3);
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();

    let mut rng_a = Xoshiro256::from_seed(42);
    let mut rng_b = Xoshiro256::from_seed(42);
    let a = mec_sample_dag(&g, &mut rng_a).unwrap();
    let b = mec_sample_dag(&g, &mut rng_b).unwrap();
    assert_eq!(a.edges(), b.edges());
}

#[test]
fn every_sample_is_a_valid_moral_member() {
    // Across many draws on 0 — 1 — 2, every sample is a fully directed acyclic
    // graph with no collider at 1 (≤ 1 parent of 1).
    let mut g = graph(3);
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();

    let mut rng = Xoshiro256::from_seed(7);
    for _ in 0..50 {
        let s = mec_sample_dag(&g, &mut rng).unwrap();
        assert!(is_fully_directed_dag(&s));
        assert!(s.parents(1).len() <= 1, "no unshielded collider at 1");
    }
}

#[test]
fn sampling_covers_the_whole_class() {
    // The three members of 0 — 1 — 2 are all reachable; over enough draws each
    // appears at least once.
    let mut g = graph(3);
    g.add_undirected(0, 1).unwrap();
    g.add_undirected(1, 2).unwrap();

    let mut rng = Xoshiro256::from_seed(123);
    let mut seen: std::collections::BTreeSet<Vec<(usize, usize)>> =
        std::collections::BTreeSet::new();
    for _ in 0..200 {
        let s = mec_sample_dag(&g, &mut rng).unwrap();
        seen.insert(s.arcs());
    }
    assert_eq!(seen.len(), 3, "all three class members should be sampled");
}

// --- error paths ------------------------------------------------------------

#[test]
fn bidirected_edge_is_not_a_cpdag() {
    let mut g = graph(2);
    g.add_bidirected(0, 1).unwrap();
    assert_eq!(mec_size(&g), Err(MecError::NotACpdag));
    assert_eq!(representative_dag(&g).err(), Some(MecError::NotACpdag));
}

#[test]
fn cyclic_arc_projection_is_not_a_dag() {
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(2, 0).unwrap();
    assert_eq!(mec_size(&g), Err(MecError::NotAcyclic));
}

#[test]
fn representative_of_cyclic_graph_errors_not_acyclic() {
    // representative_dag mirrors mec_size's errors: a cyclic arc projection
    // (fully directed but not a DAG) yields NotAcyclic, not a clone.
    let mut g = graph(3);
    g.add_arc(0, 1).unwrap();
    g.add_arc(1, 2).unwrap();
    g.add_arc(2, 0).unwrap();
    assert_eq!(representative_dag(&g).err(), Some(MecError::NotAcyclic));
}

#[test]
fn class_larger_than_the_bound_is_refused() {
    // 17 disjoint undirected edges → class size 2^17 = 131072 > the bound, caught
    // by the product check before enumeration runs away.
    let edges = 17;
    let mut g = graph(edges * 2);
    for i in 0..edges {
        g.add_undirected(2 * i, 2 * i + 1).unwrap();
    }
    assert_eq!(
        mec_size(&g),
        Err(MecError::ClassTooLarge {
            bound: MEC_ENUM_BOUND
        })
    );
}
