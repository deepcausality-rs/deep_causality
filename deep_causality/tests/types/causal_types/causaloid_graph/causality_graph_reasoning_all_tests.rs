/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::{test_utils, test_utils_graph};
use deep_causality::*;

#[test]
fn test_graph_evaluate_success() {
    let g = test_utils_graph::build_multi_cause_graph();
    let root_index = 0;

    // Create an initial effect to be applied to the root node
    let effect = PropagatingEffect::from_numerical(0.99); // A value that will activate all nodes
    // Here we evaluate the effect on the root node only,
    let res = g.evaluate_shortest_path_between_causes(root_index, root_index, &effect);
    // dbg!(&res);
    assert!(res.is_ok());
    // The root node returns Boolean(true) because its causal function evaluates to true w.r.t. to effect
}

#[test]
fn test_graph_evaluate_error_root_not_found() {
    // Test case 1: Graph has no root
    let mut g = CausaloidGraph::<BaseCausaloid<f64, bool>>::new(0);

    assert!(g.is_empty());
    assert!(!g.contains_root_causaloid());
    g.freeze();
    assert!(g.is_frozen());

    let root_index = 0;
    let effect = PropagatingEffect::from_numerical(0.99);
    let res = g.evaluate_single_cause(root_index, &effect);
    dbg!(&res);

    assert!(res.is_err());
    assert!(
        res.error
            .unwrap()
            .0
            .contains("Causaloid with index 0 not found in graph"),
    );
}

#[test]
fn test_graph_evaluate_error_not_frozen() {
    let mut g = CausaloidGraph::<BaseCausaloid<f64, bool>>::new(0);

    assert!(g.is_empty());
    assert!(!g.contains_root_causaloid());

    let root_id = 0;
    let root_causaloid = test_utils::get_test_causaloid_deterministic(root_id);
    g.add_root_causaloid(root_causaloid).unwrap();
    // DO NOT call g.freeze()
    assert!(!g.is_frozen());

    let effect = PropagatingEffect::from_numerical(0.99);
    let res = g.evaluate_subgraph_from_cause(root_id as usize, &effect);
    dbg!(&res);

    assert!(res.is_err());
    assert!(
        res.error
            .unwrap()
            .0
            .contains("Graph is not frozen. Call freeze() first"),
    );
}

#[test]
fn test_graph_evaluate_error_no_start_index() {
    let mut g = CausaloidGraph::<BaseCausaloid<f64, bool>>::new(0);

    assert!(g.is_empty());
    assert!(!g.contains_root_causaloid());

    let root_id = 0;
    let root_causaloid = test_utils::get_test_causaloid_deterministic(root_id);
    g.add_root_causaloid(root_causaloid).unwrap();
    g.freeze();
    assert!(g.is_frozen());

    let false_start_idx = 890;

    let effect = PropagatingEffect::from_numerical(0.99);
    let res = g.evaluate_subgraph_from_cause(false_start_idx, &effect);
    dbg!(&res);

    assert!(res.is_err());
    assert!(
        res.error
            .unwrap()
            .0
            .contains("Graph does not contain start causaloid with index"),
    );
}
