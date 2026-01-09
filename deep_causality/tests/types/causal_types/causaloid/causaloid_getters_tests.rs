/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_graph;
use deep_causality::*;
use std::sync::Arc;
use std::sync::RwLock;

#[test]
fn test_causaloid_getters_singleton() {
    let id = 1;
    let description = "singleton getter test";

    fn causal_fn(_input: ()) -> PropagatingEffect<bool> {
        PropagatingEffect::from_value(true)
    }

    let causaloid = BaseCausaloid::<(), bool>::new(id, causal_fn, description);

    // Test all getters for Singleton
    assert_eq!(causaloid.id(), id);
    assert!(matches!(causaloid.causal_type(), CausaloidType::Singleton));
    assert!(causaloid.causal_fn().is_some());
    assert!(causaloid.context_causal_fn().is_none());
    assert!(causaloid.context().is_none());
    assert!(causaloid.causal_collection().is_none());
    assert!(causaloid.causal_graph().is_none());
    assert_eq!(causaloid.description(), description);
    assert!(causaloid.coll_aggregate_logic().is_none());
    assert!(causaloid.coll_threshold_value().is_none());
}

#[test]
fn test_causaloid_getters_contextual() {
    let id = 2;
    let description = "contextual getter test";
    let context = BaseContext::with_capacity(99, "test_ctx", 10);

    fn context_fn(
        _obs: EffectValue<()>,
        _state: (),
        _ctx: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        PropagatingProcess::pure(true)
    }

    let causaloid = BaseCausaloid::<(), bool>::new_with_context(
        id,
        context_fn,
        Arc::new(RwLock::new(context)),
        description,
    );

    // Test all getters for Contextual Singleton
    assert_eq!(causaloid.id(), id);
    assert!(matches!(causaloid.causal_type(), CausaloidType::Singleton));
    assert!(causaloid.causal_fn().is_none());
    assert!(causaloid.context_causal_fn().is_some());
    assert!(causaloid.context().is_some());
    assert!(causaloid.causal_collection().is_none());
    assert!(causaloid.causal_graph().is_none());
    assert_eq!(causaloid.description(), description);
}

#[test]
fn test_causaloid_getters_collection() {
    let id = 3;
    let description = "collection getter test";

    // Create dummy children
    fn causal_fn(_input: ()) -> PropagatingEffect<bool> {
        PropagatingEffect::from_value(true)
    }
    let c1 = BaseCausaloid::<(), bool>::new(10, causal_fn, "c1");
    let coll = Arc::new(vec![c1]);
    let agg = AggregateLogic::All;
    let thresh = 0.5;

    let causaloid = BaseCausaloid::<(), bool>::from_causal_collection(
        id,
        coll.clone(),
        description,
        agg,
        thresh,
    );

    // Test all getters for Collection
    assert_eq!(causaloid.id(), id);
    assert!(matches!(causaloid.causal_type(), CausaloidType::Collection));
    assert!(causaloid.causal_fn().is_none());
    assert!(causaloid.context_causal_fn().is_none());
    assert!(causaloid.context().is_none());
    assert!(causaloid.causal_collection().is_some());
    // Verify collection content roughly
    assert_eq!(causaloid.causal_collection().unwrap().len(), 1);

    assert!(causaloid.causal_graph().is_none());
    assert_eq!(causaloid.description(), description);

    assert!(causaloid.coll_aggregate_logic().is_some());
    assert!(matches!(
        causaloid.coll_aggregate_logic().unwrap(),
        AggregateLogic::All
    ));

    assert!(causaloid.coll_threshold_value().is_some());
    assert_eq!(*causaloid.coll_threshold_value().unwrap(), thresh);
}

#[test]
fn test_causaloid_getters_graph() {
    let id = 4;
    let description = "graph getter test";

    let graph = test_utils_graph::build_multi_cause_graph();
    let causaloid: BaseCausaloid<f64, f64> =
        Causaloid::from_causal_graph(id, description, Arc::new(graph));

    // Test all getters for Graph
    assert_eq!(causaloid.id(), id);
    assert!(matches!(causaloid.causal_type(), CausaloidType::Graph));
    assert!(causaloid.causal_fn().is_none());
    assert!(causaloid.context_causal_fn().is_none());
    assert!(causaloid.context().is_none());
    assert!(causaloid.causal_collection().is_none());
    assert!(causaloid.causal_graph().is_some());
    assert_eq!(causaloid.description(), description);
    assert!(causaloid.coll_aggregate_logic().is_none());
    assert!(causaloid.coll_threshold_value().is_none());
}
