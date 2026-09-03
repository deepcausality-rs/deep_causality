/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![cfg(feature = "qcm")]

//! The C₃-exclusion criterion, tested against van der Lugt & Lorenz (arXiv:2508.11762) rather
//! than against this implementation.
//!
//! Three anchors, none of them a reading of the code under test:
//!
//! - Example 2.12 states `C₃` in words: two commuting CNOTs, `A₃ ↛ B₁`, `A₁ ↛ B₃`, and "influence
//!   between all other systems". Seven edges.
//! - Definition 3.1 quantifies over every choice of three inputs and three outputs, so relabelling
//!   must not matter and embedding in a larger relation must not hide it.
//! - Theorem 4.9(v) is an independent characterisation of the same property, "algorithmically the
//!   most straightforward to verify from the relation", and the brute-force test below holds the
//!   search to it over every relation on three inputs and three outputs.

use deep_causality::utils_test::test_utils;
use deep_causality::{CausableGraph, CausaloidGraph};
use deep_causality_quantum::{CausalStructure, QuantumErrorEnum};

/// A relation on inputs `{0, 1, 2}` and outputs `{3, 4, 5}` from its edge list.
fn relation(edges: &[(usize, usize)]) -> CausalStructure {
    let mut g = CausalStructure::new(&[0, 1, 2], &[3, 4, 5]);
    for &(i, o) in edges {
        g.add_influence(i, o);
    }
    g
}

/// `C₃` as Example 2.12 states it, with inputs `A₁..A₃ = 0..2` and outputs `B₁..B₃ = 3..5`:
/// every pair influences except `A₁ ↛ B₃` and `A₃ ↛ B₁`.
fn paper_c3() -> CausalStructure {
    relation(&[(0, 3), (0, 4), (1, 3), (1, 4), (1, 5), (2, 4), (2, 5)])
}

/// The bipartite 6-cycle, `K₃,₃` minus a perfect matching. Not `C₃`.
fn six_cycle() -> CausalStructure {
    relation(&[(0, 4), (0, 5), (1, 3), (1, 5), (2, 3), (2, 4)])
}

// ---------------------------------------------------------------------------
// The two canonical cases, both read off the paper.
// ---------------------------------------------------------------------------

#[test]
fn test_the_two_cnot_structure_is_c3_and_is_rejected() {
    let g = paper_c3();
    let (rows, cols) = g.find_c3().expect("Example 2.12's structure is C₃");
    assert_eq!(rows, [0, 1, 2]);
    assert_eq!(cols, [3, 4, 5]);

    let err = g.check_c3_exclusion().unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::NotFaithfullyRepresentable(_)
    ));
}

#[test]
fn test_the_six_cycle_is_not_c3_and_is_accepted() {
    // Six edges cannot be a seven-edge relation, and Theorem 4.9(v) admits this one directly: any
    // two outputs share exactly one parent, so overlapping output pairs have disjoint parent sets.
    // This is the shape the check once looked for, so it is the regression that matters most.
    let g = six_cycle();
    assert!(g.find_c3().is_none());
    assert!(g.check_c3_exclusion().is_ok());
}

// ---------------------------------------------------------------------------
// Definition 3.1 quantifies over every labelling and every embedding.
// ---------------------------------------------------------------------------

#[test]
fn test_c3_is_found_under_relabelling() {
    // Input 1 reaches every output and output 5 is reached by every input; the two non-edges
    // (0, 4) and (2, 3) share neither a row nor a column. Isomorphic to C₃, differently labelled.
    let g = relation(&[(0, 3), (0, 5), (1, 3), (1, 4), (1, 5), (2, 4), (2, 5)]);
    assert!(g.find_c3().is_some());
    assert!(g.check_c3_exclusion().is_err());
}

#[test]
fn test_c3_is_found_inside_a_larger_relation() {
    // Four inputs and four outputs, with Example 2.12's C₃ on {1, 2, 3} × {5, 6, 7}. The padding
    // input 0 and output 4 carry a single edge between them, so they enlarge the search space without
    // forming a second copy: any block containing input 0 has a row of degree at most one.
    let mut g = CausalStructure::new(&[0, 1, 2, 3], &[4, 5, 6, 7]);
    g.add_influence(0, 4);
    for &(i, o) in &[(1, 5), (1, 6), (2, 5), (2, 6), (2, 7), (3, 6), (3, 7)] {
        g.add_influence(i, o);
    }
    let (rows, cols) = g.find_c3().expect("the embedded C₃ must be found");
    assert_eq!(rows, [1, 2, 3]);
    assert_eq!(cols, [5, 6, 7]);
}

#[test]
fn test_seven_edges_with_non_edges_in_one_row_is_not_c3() {
    // Seven edges but both non-edges on input 0, so its degree is one and no relabelling reaches
    // Example 2.12's structure. This pins the "distinct rows and columns" half of the definition.
    let g = relation(&[(0, 3), (1, 3), (1, 4), (1, 5), (2, 3), (2, 4), (2, 5)]);
    assert!(g.find_c3().is_none());
    assert!(g.check_c3_exclusion().is_ok());
}

#[test]
fn test_complete_bipartite_is_c3_free() {
    // K₃,₃: nine edges, no non-edge at all, so it is not C₃ and contains nothing else to check.
    let g = relation(&[
        (0, 3),
        (0, 4),
        (0, 5),
        (1, 3),
        (1, 4),
        (1, 5),
        (2, 3),
        (2, 4),
        (2, 5),
    ]);
    assert!(g.check_c3_exclusion().is_ok());
}

#[test]
fn test_a_monotone_relation_is_c3_free() {
    // Input i influences every output ≥ i. Every 3×3 block is triangular, with a row of degree
    // three and a row of degree one, so none is C₃.
    let mut g = CausalStructure::new(&[0, 1, 2, 3], &[0, 1, 2, 3]);
    for i in 0..4 {
        for o in i..4 {
            g.add_influence(i, o);
        }
    }
    assert!(g.find_c3().is_none());
    assert!(g.check_c3_exclusion().is_ok());
}

#[test]
fn test_too_few_systems_cannot_contain_c3() {
    let mut g = CausalStructure::new(&[0, 1], &[2, 3]);
    g.add_influence(0, 3).add_influence(1, 2);
    assert!(g.find_c3().is_none());
    assert!(g.check_c3_exclusion().is_ok());
}

// ---------------------------------------------------------------------------
// Theorem 4.9(v), as an independent oracle over every 3 × 3 relation.
// ---------------------------------------------------------------------------

/// Theorem 4.9(v): `G` satisfies the C₃-exclusion property iff for all outputs `b₁, b₂, b₃` the
/// parent sets `p({b₁, b₂})` and `p({b₂, b₃})` are disjoint or nested, where `p(S)` is the set of
/// inputs influencing every output in `S`.
fn satisfies_c3_ep_by_theorem_4_9_v(g: &CausalStructure) -> bool {
    let parents = |outs: &[usize]| -> Vec<usize> {
        g.inputs()
            .iter()
            .copied()
            .filter(|&i| outs.iter().all(|&o| g.influences(i, o)))
            .collect()
    };
    let nested = |a: &[usize], b: &[usize]| {
        a.iter().all(|x| b.contains(x)) || b.iter().all(|x| a.contains(x))
    };
    let disjoint = |a: &[usize], b: &[usize]| a.iter().all(|x| !b.contains(x));
    let outs = g.outputs();
    for &b1 in outs {
        for &b2 in outs {
            for &b3 in outs {
                let left = parents(&[b1, b2]);
                let right = parents(&[b2, b3]);
                if !(disjoint(&left, &right) || nested(&left, &right)) {
                    return false;
                }
            }
        }
    }
    true
}

#[test]
fn test_the_search_agrees_with_theorem_4_9_v_on_every_three_by_three_relation() {
    // On a relation with exactly three inputs and three outputs, failing the C₃-exclusion property
    // means being C₃ itself, so `find_c3` must fire exactly where the theorem's condition fails.
    let mut c3_count = 0;
    for bits in 0u16..512 {
        let edges: Vec<(usize, usize)> = (0..9)
            .filter(|k| bits & (1 << k) != 0)
            .map(|k| (k / 3, 3 + k % 3))
            .collect();
        let g = relation(&edges);
        let found = g.find_c3().is_some();
        assert_eq!(
            found,
            !satisfies_c3_ep_by_theorem_4_9_v(&g),
            "disagreement with Theorem 4.9(v) on edges {edges:?}"
        );
        c3_count += usize::from(found);
    }
    // 3! · 3! labellings over an automorphism group of order two.
    assert_eq!(c3_count, 18, "there are eighteen labelled copies of C₃");
}

// ---------------------------------------------------------------------------
// Deriving the relation from a frozen graph.
// ---------------------------------------------------------------------------

#[test]
fn test_from_graph_reachability() {
    // Chain 0 → 1 → 2. Reachability from inputs {0,1} to outputs {1,2}:
    // 0 influences 1 and 2; 1 influences 1 (self) and 2.
    let mut g = CausaloidGraph::new(0);
    let n0 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(0))
        .unwrap();
    let n1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .unwrap();
    let n2 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(2))
        .unwrap();
    g.add_edge(n0, n1).unwrap();
    g.add_edge(n1, n2).unwrap();

    // A dynamic graph is rejected (sparse-id soundness guard).
    assert!(CausalStructure::from_graph_reachability(&g, &[n0, n1], &[n1, n2]).is_err());

    g.freeze();
    let cs = CausalStructure::from_graph_reachability(&g, &[n0, n1], &[n1, n2]).unwrap();
    assert!(cs.influences(n0, n1));
    assert!(cs.influences(n0, n2));
    assert!(cs.influences(n1, n2));
    assert!(cs.influences(n1, n1)); // self-reachable
    // A chain of 3 nodes cannot contain a C₃ (needs 3 inputs × 3 outputs).
    assert!(cs.check_c3_exclusion().is_ok());
}

#[test]
fn test_inputs_and_outputs_accessors() {
    // The constructor deduplicates and sorts both id lists.
    let cs = CausalStructure::new(&[3, 1, 1], &[5, 2, 2]);
    assert_eq!(cs.inputs(), &[1, 3]);
    assert_eq!(cs.outputs(), &[2, 5]);
}

#[test]
fn test_from_graph_reachability_rejects_out_of_range_id() {
    // A declared system id outside 0..number_nodes() would be silently treated as
    // an isolated node, detaching the derived structure from the graph; reject it.
    let mut g = CausaloidGraph::new(0);
    let n0 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(0))
        .unwrap();
    let n1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .unwrap();
    g.add_edge(n0, n1).unwrap();
    g.freeze();

    // Node id 99 does not exist (the graph has 2 nodes).
    let err = CausalStructure::from_graph_reachability(&g, &[n0, 99], &[n1]).unwrap_err();
    assert!(matches!(err.0, QuantumErrorEnum::CalculationError(_)));
}
