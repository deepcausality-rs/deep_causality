/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use ultragraph::*;

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
fn test_get_graph() {
    let g = get_causal_graph();

    let size = g.size();
    assert_eq!(size, 0);

    let graph = g.get_graph();
    assert_eq!(graph.number_edges(), 0);
    assert_eq!(graph.number_nodes(), 0);
}

#[test]
fn test_id() {
    let g: CustomCausaloidGraph = CausaloidGraph::new(0);
    assert_eq!(g.number_nodes(), 0);
    assert_eq!(g.number_edges(), 0);
    assert_eq!(g.id(), 0);
}
