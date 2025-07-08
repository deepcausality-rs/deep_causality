/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::{test_utils, test_utils_graph};
use deep_causality::*;

#[test]
fn test_evaluate_shortest_path_between_causes() {
    // Reasons over a linear graph: root(0) -> A(1) -> B(2) ...
    let (g, _data) = test_utils_graph::get_small_linear_graph_and_data();
    let total_nodes = g.number_nodes() as f64; // 11 nodes total (root + 10)

    // 1. Before evaluation, assert that active status is unknown (returns Err)
    assert!(g.percent_active().is_err());
    assert!(g.number_active().is_err());
    assert!(g.all_active().is_err());

    let evidence = Evidence::Numerical(0.99);

    // 2. Evaluate the full path from the first to the last node.
    let res = g.evaluate_shortest_path_between_causes(0, 10, &evidence);
    dbg!(&res);
    assert!(res.is_ok());

    let res = res.unwrap();
    assert_eq!(res, PropagatingEffect::Deterministic(true));

    // 3. After evaluation, all nodes on the path are active.
    // The entire graph should now be 100% active.
    assert_eq!(g.number_active().unwrap(), total_nodes);
    assert_eq!(g.percent_active().unwrap(), 100.0);
    assert!(g.all_active().unwrap());
}

#[test]
fn test_shortest_path_error_conditions() {
    let evidence = Evidence::Numerical(0.99);

    // Error: Graph is not frozen
    let (mut g, _) = test_utils_graph::get_small_linear_graph_and_data();
    g.unfreeze();
    let res = g.evaluate_shortest_path_between_causes(0, 1, &evidence);
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().to_string(),
        "CausalityError: Graph is not frozen. Call freeze() first"
    );

    // Setup for remaining tests
    let (g, _) = test_utils_graph::get_small_linear_graph_and_data();

    // Error: Start node does not exist. The underlying pathfinder returns a generic error.
    let res = g.evaluate_shortest_path_between_causes(99, 1, &evidence);
    assert!(res.is_err());

    dbg!(&res);
    assert!(res.unwrap_err().to_string().contains("No path found"));

    // Error: Stop node does not exist
    let res = g.evaluate_shortest_path_between_causes(1, 99, &evidence);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("No path found"));

    // Error: No path found between nodes
    let mut g_disconnected = CausaloidGraph::new(0);
    g_disconnected
        .add_causaloid(test_utils::get_test_causaloid())
        .unwrap(); // index 0
    g_disconnected
        .add_causaloid(test_utils::get_test_causaloid())
        .unwrap(); // index 1
    g_disconnected.freeze();

    let res = g_disconnected.evaluate_shortest_path_between_causes(0, 1, &evidence);
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("No path found"));
}

#[test]
fn test_shortest_path_on_single_node() {
    // Evaluating a "path" where start and stop are the same should just evaluate the single node.
    let (g, _) = test_utils_graph::get_small_linear_graph_and_data();
    let evidence = Evidence::Numerical(0.99);

    let res = g
        .evaluate_shortest_path_between_causes(5, 5, &evidence)
        .unwrap();
    assert_eq!(res, PropagatingEffect::Deterministic(true));

    // Check that only that single node became active.
    let node = g.get_causaloid(5).unwrap();
    assert!(node.is_active().unwrap());

    // The rest of the graph remains unevaluated, so checking the whole graph's
    // active percentage will correctly return an error.
    assert!(g.percent_active().is_err());
}
