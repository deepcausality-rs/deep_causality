/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Direct tests for the `CausableGraph::get_shortest_path` default method,
//! exercising the early-return guards that the higher-level reasoning APIs do
//! not reach on their own.

use deep_causality::utils_test::{test_utils, test_utils_graph};
use deep_causality::*;

#[test]
fn test_get_shortest_path_identical_start_stop_errors() {
    // start_index == stop_index returns an explicit error before touching the graph.
    let g = test_utils_graph::build_linear_graph(4);

    let res = g.get_shortest_path(2, 2);
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert!(
        err.to_string().contains("Start and Stop node identical"),
        "unexpected error: {err}"
    );
}

#[test]
fn test_get_shortest_path_on_unfrozen_graph_errors() {
    // On a dynamic (unfrozen) graph the underlying `shortest_path` returns
    // `GraphError::GraphNotFrozen`, hitting the `Err(e)` mapping arm.
    let mut g = CausaloidGraph::new(0);
    let i0 = g
        .add_root_causaloid(test_utils::get_test_causaloid_deterministic(0))
        .expect("root");
    let i1 = g
        .add_causaloid(test_utils::get_test_causaloid_deterministic(1))
        .expect("n1");
    g.add_edge(i0, i1).expect("edge");
    // Deliberately NOT frozen.

    let res = g.get_shortest_path(i0, i1);
    assert!(res.is_err());
}

#[test]
fn test_get_shortest_path_success_returns_path() {
    // A successful lookup on a frozen linear graph returns the node path.
    let g = test_utils_graph::build_linear_graph(5);

    let res = g.get_shortest_path(0, 4);
    assert!(res.is_ok());
    let path = res.unwrap();
    assert_eq!(path.first(), Some(&0));
    assert_eq!(path.last(), Some(&4));
}
