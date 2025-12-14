/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_graph;
use deep_causality::*;
use std::sync::Arc;

#[test]
fn test_causaloid_new_singleton() {
    let id = 1;
    let description = "singleton test";

    fn causal_fn(_input: ()) -> PropagatingEffect<bool> {
        PropagatingEffect::from_value(true)
    }

    let causaloid = BaseCausaloid::<(), bool>::new(id, causal_fn, description);
    assert_eq!(causaloid.id(), id);
    assert!(causaloid.is_singleton());
    assert!(causaloid.causal_collection().is_none());
    assert!(causaloid.causal_graph().is_none());
}

#[test]
fn test_causaloid_collection() {
    fn causal_fn(_input: ()) -> PropagatingEffect<bool> {
        PropagatingEffect::from_value(true)
    }

    let c1 = BaseCausaloid::<(), bool>::new(1, causal_fn, "c1");
    let c2 = BaseCausaloid::<(), bool>::new(2, causal_fn, "c2");

    let coll = Arc::new(vec![c1, c2]);

    let c_coll = BaseCausaloid::<(), bool>::from_causal_collection(
        3,
        coll,
        "collection",
        AggregateLogic::All,
        0.0,
    );

    assert_eq!(c_coll.id(), 3);
    assert!(!c_coll.is_singleton());
    assert!(c_coll.causal_collection().is_some());
    assert!(c_coll.causal_graph().is_none());
}

#[test]
fn test_causaloid_graph() {
    // Use existing test utility to build a valid causal graph
    let graph = test_utils_graph::build_multi_cause_graph();

    let c_graph: BaseCausaloid<f64, f64> =
        Causaloid::from_causal_graph(4, "graph from utility", Arc::new(graph));

    assert_eq!(c_graph.id(), 4);
    assert!(!c_graph.is_singleton());
    assert!(c_graph.causal_collection().is_none());
    assert!(c_graph.causal_graph().is_some());
}
