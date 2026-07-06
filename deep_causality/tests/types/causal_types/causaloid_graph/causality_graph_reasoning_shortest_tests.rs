/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::{test_utils, test_utils_graph};
use deep_causality::*;

#[test]
fn test_evaluate_shortest_path_between_causes() {
    // Reasons over a linear graph: root(0) -> A(1) -> B(2) ...
    let g = test_utils_graph::build_linear_graph(6); // K needs to be at least +1 than stop index

    let effect = PropagatingEffect::from_value(0.99);

    // 2. Evaluate the full path from the first to the last node.
    let res = g.evaluate_shortest_path_between_causes(0, 5, &effect);
    dbg!(&res);
    assert!(res.is_ok());

    assert_eq!(res.value(), Some(&1.0));
}

#[test]
fn test_shortest_path_on_single_node() {
    // Evaluating a "path" where start and stop are the same should just evaluate the single node.
    let g = test_utils_graph::build_linear_graph(7); // K needs to be at least +1 than stop index
    let effect = PropagatingEffect::from_value(0.99);

    let res = g.evaluate_shortest_path_between_causes(5, 5, &effect);
    dbg!(&res);
    assert!(res.is_ok());

    assert_eq!(res.value(), Some(&1.0));
}

#[test]
fn test_shortest_path_short_circuits_on_a_failing_node() {
    // 0 -> 1(error) -> 2: walking the path aborts at the failing node.
    let mut g = CausaloidGraph::new(0);
    let i0 = g
        .add_root_causaloid(test_utils::get_test_causaloid_deterministic_input_output())
        .expect("root");
    let i1 = g
        .add_causaloid(test_utils::get_test_error_causaloid())
        .expect("err");
    let i2 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic_input_output())
        .expect("n2");
    g.add_edge(i0, i1).expect("edge");
    g.add_edge(i1, i2).expect("edge");
    g.freeze();

    let res = g.evaluate_shortest_path_between_causes(i0, i2, &PropagatingEffect::from_value(true));
    assert!(res.is_err());
    assert!(res.error().unwrap().to_string().contains("Test error"));
}

fn relay_to_9(_obs: bool) -> PropagatingEffect<bool> {
    PropagatingEffect::from_effect_value(EffectValue::RelayTo(
        9,
        Box::new(PropagatingEffect::from_value(true)),
    ))
}

#[test]
fn test_shortest_path_returns_on_a_relay() {
    // 0 -> 1(relay) -> 2: the walk returns the relaying node's effect verbatim.
    let mut g = CausaloidGraph::new(0);
    let i0 = g
        .add_root_causaloid(test_utils::get_test_causaloid_deterministic_input_output())
        .expect("root");
    let i1 = g
        .add_causaloid(Causaloid::new(1, relay_to_9, "relayer"))
        .expect("relay");
    let i2 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic_input_output())
        .expect("n2");
    g.add_edge(i0, i1).expect("edge");
    g.add_edge(i1, i2).expect("edge");
    g.freeze();

    let res = g.evaluate_shortest_path_between_causes(i0, i2, &PropagatingEffect::from_value(true));
    assert!(res.is_ok(), "got {:?}", res.error());
    assert!(matches!(res.effect(), Some(EffectValue::RelayTo(9, _))));
}

#[test]
fn test_shortest_path_error_conditions() {
    let effect = PropagatingEffect::from_value(0.99);

    // Error: Graph is not frozen
    let mut g = test_utils_graph::build_linear_graph(3);
    g.unfreeze();
    let res = g.evaluate_shortest_path_between_causes(0, 1, &effect);
    assert!(res.is_err());
    assert!(
        res.error()
            .unwrap()
            .to_string()
            .contains("Graph is not frozen. Call freeze() first")
    );

    // Setup for remaining tests
    let g = test_utils_graph::build_linear_graph(7); // K needs to be at least +1 than stop index

    // Error: Start node does not exist. The underlying pathfinder returns a generic error.
    let res = g.evaluate_shortest_path_between_causes(99, 1, &effect);
    assert!(res.is_err());

    dbg!(&res);
    assert!(res.error().unwrap().to_string().contains("No path found"));
}
