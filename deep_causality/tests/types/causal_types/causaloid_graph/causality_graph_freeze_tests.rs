/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::{CausableGraph, CausaloidGraph};

#[test]
fn test_freeze_unfreeze() {
    let mut g = CausaloidGraph::new(0);

    // Add root causaloid
    let root_causaloid = test_utils::get_test_causaloid_deterministic(0);
    let root_index = g
        .add_root_causaloid(root_causaloid)
        .expect("Failed to add root index");
    let contains_root = g.contains_causaloid(root_index);
    assert!(contains_root);

    // Add causaloid A
    let causaloid = test_utils::get_test_causaloid_deterministic(1);
    let idx_a = g.add_causaloid(causaloid).expect("Failed to add causaloid");

    g.freeze();

    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    g.unfreeze();

    let causaloid = test_utils::get_test_causaloid_deterministic(2);
    let idx_b = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let res = g.add_edge(root_index, idx_b);
    assert!(res.is_ok());

    g.freeze();

    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);
}

//
// freeze_dag() — opt-in DAG enforcement
//

#[test]
fn test_freeze_dag_acyclic_ok() {
    // A simple chain a -> b is acyclic; freeze_dag must succeed and leave the graph frozen.
    let mut g = CausaloidGraph::new(0);
    let a = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("Failed to add causaloid a");
    let b = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(2))
        .expect("Failed to add causaloid b");
    g.add_edge(a, b).expect("Failed to add edge a -> b");

    let res = g.freeze_dag();

    assert!(res.is_ok());
    assert!(g.is_frozen());
}

#[test]
fn test_freeze_dag_empty_ok() {
    // An empty graph is trivially a DAG; freeze_dag must succeed and freeze it.
    let mut g: CausaloidGraph<deep_causality::BaseCausaloid<deep_causality::NumericalValue, bool>> =
        CausaloidGraph::new(0);

    let res = g.freeze_dag();

    assert!(res.is_ok());
    assert!(g.is_frozen());
}

#[test]
fn test_freeze_dag_diamond_reconvergent_ok() {
    // Reconvergent DAG (diamond): a -> b, a -> c, b -> d, c -> d.
    // It is acyclic, so freeze_dag accepts it. This documents the scope: freeze_dag
    // enforces acyclicity only; it does not reject reconvergence (joins).
    let mut g = CausaloidGraph::new(0);
    let a = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("Failed to add a");
    let b = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(2))
        .expect("Failed to add b");
    let c = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(3))
        .expect("Failed to add c");
    let d = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(4))
        .expect("Failed to add d");
    g.add_edge(a, b).expect("edge a -> b");
    g.add_edge(a, c).expect("edge a -> c");
    g.add_edge(b, d).expect("edge b -> d");
    g.add_edge(c, d).expect("edge c -> d");

    let res = g.freeze_dag();

    assert!(res.is_ok());
    assert!(g.is_frozen());
}

#[test]
fn test_freeze_dag_already_frozen_ok() {
    // Calling freeze_dag on an already-frozen acyclic graph is a no-op freeze followed by
    // a successful cycle check; the graph stays frozen.
    let mut g = CausaloidGraph::new(0);
    let a = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("Failed to add a");
    let b = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(2))
        .expect("Failed to add b");
    g.add_edge(a, b).expect("edge a -> b");

    g.freeze();
    assert!(g.is_frozen());

    let res = g.freeze_dag();

    assert!(res.is_ok());
    assert!(g.is_frozen());
}

#[test]
fn test_freeze_dag_cycle_err() {
    // A 2-cycle a -> b -> a must be rejected. The graph is rolled back to unfrozen so it is
    // never left presented as a frozen, cyclic graph.
    let mut g = CausaloidGraph::new(0);
    let a = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("Failed to add a");
    let b = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(2))
        .expect("Failed to add b");
    g.add_edge(a, b).expect("edge a -> b");
    g.add_edge(b, a).expect("edge b -> a");

    let res = g.freeze_dag();

    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert!(msg.contains("cycle"));
    // Rolled back: the graph is not left frozen.
    assert!(!g.is_frozen());
}

#[test]
fn test_freeze_dag_self_loop_err() {
    // A self-loop a -> a is a cycle and must be rejected.
    let mut g = CausaloidGraph::new(0);
    let a = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("Failed to add a");
    g.add_edge(a, a).expect("self-loop edge a -> a");

    let res = g.freeze_dag();

    assert!(res.is_err());
    assert!(!g.is_frozen());
}

#[test]
fn test_freeze_dag_rolled_back_graph_is_usable() {
    // After a failed freeze_dag the graph is dynamic again, so it can be fixed (break the
    // cycle) and frozen as a DAG on a second attempt.
    let mut g = CausaloidGraph::new(0);
    let a = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("Failed to add a");
    let b = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(2))
        .expect("Failed to add b");
    g.add_edge(a, b).expect("edge a -> b");
    g.add_edge(b, a).expect("edge b -> a");

    assert!(g.freeze_dag().is_err());
    assert!(!g.is_frozen());

    // Break the cycle, then freeze_dag must succeed.
    g.remove_edge(b, a).expect("remove edge b -> a");
    let res = g.freeze_dag();

    assert!(res.is_ok());
    assert!(g.is_frozen());
}

#[test]
fn test_freeze_unchanged_on_cyclic_graph() {
    // Additivity guarantee: the existing freeze() still accepts a cyclic graph unchanged.
    let mut g = CausaloidGraph::new(0);
    let a = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("Failed to add a");
    let b = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(2))
        .expect("Failed to add b");
    g.add_edge(a, b).expect("edge a -> b");
    g.add_edge(b, a).expect("edge b -> a");

    g.freeze();

    assert!(g.is_frozen());
}
