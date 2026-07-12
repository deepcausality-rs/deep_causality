/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::{CausableGraph, CausaloidGraph};
use deep_causality_quantum::{CausalStructure, QuantumErrorEnum};

/// The canonical `C₃` obstruction (two commuting CNOTs, `U₃`): the bipartite
/// 6-cycle `K_{3,3}` minus a perfect matching — every input relates to exactly
/// two outputs and vice versa.
fn canonical_c3() -> CausalStructure {
    let mut g = CausalStructure::new(&[0, 1, 2], &[3, 4, 5]);
    // Non-edges form the "diagonal" matching (0,3), (1,4), (2,5); all else edges.
    g.add_influence(0, 4).add_influence(0, 5);
    g.add_influence(1, 3).add_influence(1, 5);
    g.add_influence(2, 3).add_influence(2, 4);
    g
}

#[test]
fn test_c3_is_detected_and_rejected() {
    let g = canonical_c3();
    let witness = g.find_c3();
    assert!(witness.is_some());
    let (rows, cols) = witness.unwrap();
    assert_eq!(rows, [0, 1, 2]);
    assert_eq!(cols, [3, 4, 5]);

    let err = g.check_c3_exclusion().unwrap_err();
    assert!(matches!(
        err.0,
        QuantumErrorEnum::NotFaithfullyRepresentable(_)
    ));
}

#[test]
fn test_c3_free_structure_is_accepted() {
    // A "layered"/monotone relation: input i influences every output ≥ i. No
    // 3×3 block can be a 6-cycle (a full lower/upper-triangular block has a
    // row or column of degree 3 or 0).
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
fn test_complete_bipartite_is_c3_free() {
    // K_{3,3} itself (every input to every output): every 3×3 block has all
    // degrees 3, never 2 → no C₃.
    let mut g = CausalStructure::new(&[0, 1, 2], &[3, 4, 5]);
    for i in 0..3 {
        for o in 3..6 {
            g.add_influence(i, o);
        }
    }
    assert!(g.check_c3_exclusion().is_ok());
}

#[test]
fn test_too_few_systems_cannot_contain_c3() {
    let mut g = CausalStructure::new(&[0, 1], &[2, 3]);
    g.add_influence(0, 3).add_influence(1, 2);
    assert!(g.find_c3().is_none());
    assert!(g.check_c3_exclusion().is_ok());
}

#[test]
fn test_c3_up_to_relabelling() {
    // A cyclic (rather than "≠") non-edge matching still forms a 6-cycle and is
    // detected: non-edges (0,4),(1,5),(2,3).
    let mut g = CausalStructure::new(&[0, 1, 2], &[3, 4, 5]);
    g.add_influence(0, 3).add_influence(0, 5);
    g.add_influence(1, 3).add_influence(1, 4);
    g.add_influence(2, 4).add_influence(2, 5);
    assert!(g.find_c3().is_some());
    assert!(g.check_c3_exclusion().is_err());
}

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
