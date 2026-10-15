/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `MonadicCausableGraphReasoning` over a frozen graph with a removed causaloid.
//!
//! Positions are permanent: an index taken from `add_causaloid` names the same causaloid after a
//! removal and a freeze, so a live causaloid can sit at an index at or above `number_nodes()`.
//! The engine sizes its bookkeeping by `index_bound()` and reaches every such causaloid.

use deep_causality::{
    CausableGraph, CausalityError, Causaloid, CausaloidGraph, LambdaEdges,
    MonadicCausableGraphReasoning, PropagatingEffect,
};

fn add_one(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x + 1.0)
}
fn times_hundred(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x * 100.0)
}
fn double(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x * 2.0)
}
fn add_ten(x: f64) -> PropagatingEffect<f64> {
    PropagatingEffect::from_value(x + 10.0)
}
fn double_lambda(v: f64) -> f64 {
    v * 2.0
}

type Graph = CausaloidGraph<Causaloid<f64, f64, (), ()>>;

/// Causaloids 0 (`+1`), 1 (`×100`) and 2 (`×2`) with edges 0 → 1, 1 → 2 and 0 → 2; index 1
/// removed; frozen. Only the edge 0 → 2 survives the removal.
fn removed_middle_graph() -> Graph {
    let mut g = CausaloidGraph::new(0);
    let i0 = g.add_causaloid(Causaloid::new(0, add_one, "0")).unwrap();
    let i1 = g
        .add_causaloid(Causaloid::new(1, times_hundred, "1"))
        .unwrap();
    let i2 = g.add_causaloid(Causaloid::new(2, double, "2")).unwrap();
    assert_eq!([i0, i1, i2], [0, 1, 2]);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(0, 2).unwrap();
    g.remove_causaloid(1).unwrap();
    g.freeze();
    g
}

/// The chain 0 (`+1`) → 1 (`×100`) → 2 (`×2`) → 3 (`+10`) with 0 → 2 added and index 1 removed;
/// frozen. The live causaloid 3 sits at an index >= `number_nodes()` = 3.
fn chain_with_removed_middle() -> Graph {
    let mut g = CausaloidGraph::new(0);
    let i0 = g.add_causaloid(Causaloid::new(0, add_one, "0")).unwrap();
    let i1 = g
        .add_causaloid(Causaloid::new(1, times_hundred, "1"))
        .unwrap();
    let i2 = g.add_causaloid(Causaloid::new(2, double, "2")).unwrap();
    let i3 = g.add_causaloid(Causaloid::new(3, add_ten, "3")).unwrap();
    assert_eq!([i0, i1, i2, i3], [0, 1, 2, 3]);
    g.add_edge(0, 1).unwrap();
    g.add_edge(1, 2).unwrap();
    g.add_edge(2, 3).unwrap();
    g.add_edge(0, 2).unwrap();
    g.remove_causaloid(1).unwrap();
    g.freeze();
    g
}

#[test]
fn test_evaluation_crosses_a_removed_index() {
    let g = removed_middle_graph();
    // The index `add_causaloid` returned still names causaloid 2 after the removal and the freeze.
    assert_eq!(g.number_nodes(), 2);
    assert!(!g.contains_causaloid(1));
    assert!(g.contains_causaloid(2));
    assert_eq!(g.get_causaloid(2).map(|c| c.id()), Some(2));

    // Hand derivation: 0 turns 3 into 3 + 1 = 4; the removed 1 never fires; 2 doubles 4 to 8.
    // Through 1 the value would have been (3 + 1) · 100 · 2 = 800.
    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(3.0));
    assert!(res.is_ok(), "got {:?}", res.error());
    assert_eq!(res.value(), Some(&8.0));
}

#[test]
fn test_evaluation_starts_at_a_live_index_at_or_above_number_nodes() {
    let g = removed_middle_graph();
    assert!(2 >= g.number_nodes());

    // Hand derivation: 2 doubles 3 to 6; nothing lies below 2.
    let res = g.evaluate_subgraph_from_cause(2, &PropagatingEffect::from_value(3.0));
    assert!(res.is_ok(), "got {:?}", res.error());
    assert_eq!(res.value(), Some(&6.0));
}

#[test]
fn test_evaluation_reaches_a_causaloid_at_an_index_above_number_nodes() {
    let g = chain_with_removed_middle();
    assert_eq!(g.number_nodes(), 3);
    assert!(3 >= g.number_nodes());
    assert!(g.contains_causaloid(3));

    // Hand derivation: (3 + 1) · 2 + 10 = 18. Without causaloid 3 firing the result would be the
    // output of 2, which is 8.
    let res = g.evaluate_subgraph_from_cause(0, &PropagatingEffect::from_value(3.0));
    assert!(res.is_ok(), "got {:?}", res.error());
    assert_eq!(res.value(), Some(&18.0));
}

#[test]
fn test_evaluation_from_a_removed_index_is_causaloid_not_found() {
    let g = removed_middle_graph();
    let res = g.evaluate_subgraph_from_cause(1, &PropagatingEffect::from_value(3.0));
    assert_eq!(res.error(), Some(&CausalityError::CausaloidNotFound(1)));
}

#[test]
fn test_a_decorated_edge_keeps_its_decoration_after_a_removal() {
    // Causaloids 0 (`+1`), 1 (`×100`), 2 (`+1`), 3 (`+10`); edges 0 → 2 and 2 → 3; index 1
    // removed; frozen. The Λ on 2 → 3 is keyed by the indices `add_causaloid` returned.
    let mut g: Graph = CausaloidGraph::new(0);
    g.add_causaloid(Causaloid::new(0, add_one, "0")).unwrap();
    g.add_causaloid(Causaloid::new(1, times_hundred, "1"))
        .unwrap();
    let i2 = g.add_causaloid(Causaloid::new(2, add_one, "2")).unwrap();
    let i3 = g.add_causaloid(Causaloid::new(3, add_ten, "3")).unwrap();
    g.add_edge(0, i2).unwrap();
    g.add_edge(i2, i3).unwrap();
    g.remove_causaloid(1).unwrap();
    g.freeze();
    let lambdas = LambdaEdges::new().with_lambda(i2, i3, double_lambda);

    // Hand derivation: 2 turns 3 into 4; the Λ on 2 → 3 doubles it to 8; 3 adds ten: 18.
    let decorated = g.evaluate_subgraph_from_cause_with_lambda_edges(
        i2,
        &PropagatingEffect::from_value(3.0),
        &lambdas,
    );
    assert!(decorated.is_ok(), "got {:?}", decorated.error());
    assert_eq!(decorated.value(), Some(&18.0));

    // Undecorated the same walk gives 4 + 10 = 14: the value arriving at 3 was doubled by the Λ.
    let plain = g.evaluate_subgraph_from_cause(i2, &PropagatingEffect::from_value(3.0));
    assert!(plain.is_ok(), "got {:?}", plain.error());
    assert_eq!(plain.value(), Some(&14.0));
}
