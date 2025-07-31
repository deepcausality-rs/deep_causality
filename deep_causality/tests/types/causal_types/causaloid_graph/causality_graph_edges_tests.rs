/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use ultragraph::*;

use deep_causality::utils_test::test_utils;

// Custom type alias
type CustomCausaloidGraph = CausaloidGraph<
    Causaloid<
        Data<NumberType>,
        EuclideanSpace,
        EuclideanTime,
        EuclideanSpacetime,
        BaseSymbol,
        FloatType,
        FloatType,
    >,
>;

fn get_causal_graph() -> BaseCausalGraph {
    let g: BaseCausalGraph = CausaloidGraph::new(0);
    g
}

#[test]
fn test_new() {
    let g: CustomCausaloidGraph = CausaloidGraph::new(0);
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_new_with_capacity() {
    let g: CustomCausaloidGraph = CausaloidGraph::new_with_capacity(0, 10);
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_default() {
    let g: CustomCausaloidGraph = CausaloidGraph::default();
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
}

#[test]
fn test_add_edge() {
    let mut g = get_causal_graph();
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let idx_a = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid_deterministic();
    let idx_b = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let res = g.add_edge(idx_a, idx_b);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert!(contains_edge);
}

#[test]
fn test_add_edg_with_weight() {
    let mut g = get_causal_graph();
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let idx_a = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid_deterministic();
    let idx_b = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let weight = 1;
    let res = g.add_edg_with_weight(idx_a, idx_b, weight);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert!(contains_edge);
}

#[test]
fn test_remove_edge() {
    let mut g = get_causal_graph();
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let idx_a = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains_a = g.contains_causaloid(idx_a);
    assert!(contains_a);

    let causaloid = test_utils::get_test_causaloid_deterministic();
    let idx_b = g.add_causaloid(causaloid).expect("Failed to add causaloid");
    let contains_b = g.contains_causaloid(idx_b);
    assert!(contains_b);

    let res = g.add_edge(idx_a, idx_b);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert!(contains_edge);

    let res = g.remove_edge(idx_a, idx_b);
    assert!(res.is_ok());

    let contains_edge = g.contains_edge(idx_a, idx_b);
    assert!(!contains_edge);
}

#[test]
fn test_get_graph() {
    let g = get_causal_graph();

    let size = g.size();
    assert_eq!(size, 0);

    let graph = g.get_graph();
    assert_eq!(graph.number_edges(), 0);
    assert_eq!(graph.number_nodes(), 0);
}
