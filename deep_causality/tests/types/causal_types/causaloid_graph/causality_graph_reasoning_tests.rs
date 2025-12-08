/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{
    CausableGraph, Causaloid, CausaloidGraph, MonadicCausableGraphReasoning, PropagatingEffect,
};
use deep_causality_core::{EffectValue};

type TestGraph = CausaloidGraph<Causaloid<bool, bool, (), ()>>;

fn create_graph() -> TestGraph {
    CausaloidGraph::new(1)
}

// Custom identity function for tests
fn boolean_true(_: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(true)
}

fn boolean_false(_: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(false)
}

fn identity(obs: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(obs)
}

fn inverter(obs: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::pure(!obs)
}

fn relayer(_: bool) -> PropagatingEffect<bool> {
    // Relay to node 2 with value false
    let mut eff = PropagatingEffect::pure(false);
    eff.value = EffectValue::RelayTo(2, Box::new(PropagatingEffect::pure(false)));
    eff
}

#[test]
fn test_evaluate_single_cause() {
    let mut graph = create_graph();
    
    // Create local causaloids to match TestGraph type
    let c1 = Causaloid::new(1, boolean_true, "True");
    let c2 = Causaloid::new(2, boolean_false, "False");

    let idx1 = graph.add_causaloid(c1).unwrap();
    let idx2 = graph.add_causaloid(c2).unwrap();

    // Verify error if not frozen
    let res = graph.evaluate_single_cause(idx1, &PropagatingEffect::pure(true));
    assert!(res.is_err());
    assert!(res
        .error
        .unwrap()
        .to_string()
        .contains("Graph is not frozen"));

    graph.freeze();

    // Verify success true
    let res1 = graph.evaluate_single_cause(idx1, &PropagatingEffect::pure(false));
    assert!(res1.value.into_value().unwrap());

    // Verify success false
    let res2 = graph.evaluate_single_cause(idx2, &PropagatingEffect::pure(true));
    assert!(!res2.value.into_value().unwrap());

    // Verify index not found
    let res_err = graph.evaluate_single_cause(999, &PropagatingEffect::pure(true));
    assert!(res_err.is_err());
    assert!(res_err.error.unwrap().to_string().contains("not found"));
}

#[test]
fn test_evaluate_subgraph_from_cause() {
    let mut graph = create_graph();
    // Logic:
    // N0: Identity (returns input)
    // N1: Not (inverts input)
    // N2: Identity
    // Edges: N0 -> N1 -> N2

    let c_ident = Causaloid::new(1, identity, "Identity");
    let c_invert = Causaloid::new(2, inverter, "Inverter");

    let idx0 = graph.add_causaloid(c_ident.clone()).unwrap();
    let idx1 = graph.add_causaloid(c_invert.clone()).unwrap();
    let idx2 = graph.add_causaloid(c_ident.clone()).unwrap();
    let idx3 = graph.add_causaloid(c_invert.clone()).unwrap();

    // Linear chain: 0 -> 1 -> 2 -> 3
    // Flow: True -> (Id) True -> (Inv) False -> (Id) False -> (Inv) True
    graph.add_edge(idx0, idx1).unwrap();
    graph.add_edge(idx1, idx2).unwrap();
    graph.add_edge(idx2, idx3).unwrap();

    graph.freeze();

    let res = graph.evaluate_subgraph_from_cause(idx0, &PropagatingEffect::pure(true));
    assert!(res.value.into_value().unwrap()); // Expect True

    // Check RelayTo logic
    // Create new graph with Relay
    let mut relay_graph = create_graph();
    let r0 = c_ident.clone();
    let c_relay = Causaloid::new(3, relayer, "Relayer");
    // R2 is inverter
    let c_target = c_invert.clone();

    let ridx0 = relay_graph.add_causaloid(r0).unwrap();
    let ridx1 = relay_graph.add_causaloid(c_relay).unwrap();
    let ridx2 = relay_graph.add_causaloid(c_target).unwrap(); // Index 2
    
    assert_eq!(ridx2, 2, "Assumed index 2 for relay target");

    // Edge 0 -> 1. NO edge to 2. Relay should jump from 1 to 2.
    relay_graph.add_edge(ridx0, ridx1).unwrap();

    relay_graph.freeze();

    // Flow: [0] True -> [1] RelayTo(2, False) -> [2] Invert(False) -> True
    let r_res = relay_graph.evaluate_subgraph_from_cause(ridx0, &PropagatingEffect::pure(true));
    
    if r_res.is_err() {
        panic!("Evaluate failed: {:?}", r_res.error);
    }
    
    if let EffectValue::Value(val) = r_res.value {
        assert!(val, "Expected True from RelayTo flow");
    } else {
        panic!("Expected Value(True) but got {:?}", r_res.value);
    }
}

#[test]
fn test_evaluate_shortest_path_between_causes() {
    let _graph = create_graph(); 

    // n0(T) -> n1(!T=F) -> n2(F)  [Shortest]
    // n0(T) -> n3(T) -> n4(T) -> n2(T) [Longer]

    // We need to re-make graph with distinct logic
    let mut logic_graph = create_graph();
    
    let c_id = Causaloid::new(1, identity, "Id");
    let c_invert = Causaloid::new(2, inverter, "Inverter");

    let idx0 = logic_graph.add_causaloid(c_id.clone()).unwrap(); // Id
    let idx1 = logic_graph.add_causaloid(c_invert.clone()).unwrap(); // Invert
    let idx2 = logic_graph.add_causaloid(c_id.clone()).unwrap(); // Id
    let idx3 = logic_graph.add_causaloid(c_id.clone()).unwrap(); // Id
    let idx4 = logic_graph.add_causaloid(c_id.clone()).unwrap(); // Id

    logic_graph.add_edge(idx0, idx1).unwrap();
    logic_graph.add_edge(idx1, idx2).unwrap();

    logic_graph.add_edge(idx0, idx3).unwrap();
    logic_graph.add_edge(idx3, idx4).unwrap();
    logic_graph.add_edge(idx4, idx2).unwrap();

    logic_graph.freeze();

    let res_logic = logic_graph.evaluate_shortest_path_between_causes(
        idx0,
        idx2,
        &PropagatingEffect::pure(true),
    );
    
    // Shortest path: 0->1->2. 1 inverts True -> False. Result False.
    assert!(!res_logic.value.into_value().unwrap(), "Expected False from shortest path");
}
