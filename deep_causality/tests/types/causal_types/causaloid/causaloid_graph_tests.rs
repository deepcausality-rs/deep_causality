/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils::get_base_context;
use deep_causality::{
    Causable, Causaloid, IdentificationValue, MonadicCausable, PropagatingEffect,
};
use std::sync::{Arc, RwLock};

#[test]
fn test_from_causal_graph() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let (causal_graph, _) = test_utils_graph::get_small_multi_layer_cause_graph_and_data();

    let causaloid = Causaloid::from_causal_graph(id, Arc::new(causal_graph), description);
    assert!(!causaloid.is_singleton());

    let explain_res = causaloid.explain().unwrap();
    assert_eq!(
        explain_res,
        "No nodes in the graph have been evaluated or produced an explainable effect.".to_string()
    );

    // Use the new `evaluate` method.
    let effect = PropagatingEffect::Numerical(0.99);
    let res = causaloid.evaluate(&effect);
    assert!(res.is_ok());

    // The default evaluation of a graph causaloid should propagate.
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    assert!(causaloid.context().is_none());
}

#[test]
fn test_from_causal_graph_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let (causal_graph, _) = test_utils_graph::get_small_multi_layer_cause_graph_and_data();
    let context = get_base_context();

    let causaloid = Causaloid::from_causal_graph_with_context(
        id,
        Arc::new(causal_graph),
        Arc::new(RwLock::new(context)),
        description,
    );
    assert!(!causaloid.is_singleton());

    let explain_res = causaloid.explain().unwrap();
    assert_eq!(
        explain_res,
        "No nodes in the graph have been evaluated or produced an explainable effect.".to_string()
    );

    let effect = PropagatingEffect::Numerical(0.99);
    let res = causaloid.evaluate(&effect);
    assert!(res.is_ok());

    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
    assert!(causaloid.context().is_some());
}

#[test]
fn test_causal_graph_explain() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let (causal_graph, _) = test_utils_graph::get_small_multi_layer_cause_graph_and_data();

    let causaloid = Causaloid::from_causal_graph(id, Arc::new(causal_graph), description);
    assert!(!causaloid.is_singleton());

    assert!(causaloid.causal_graph().is_some());
    assert!(causaloid.causal_collection().is_none());

    // explain() on an unevaluated graph returns Ok, not Err.
    let explain_res = causaloid.explain().unwrap();
    assert_eq!(
        explain_res,
        "No nodes in the graph have been evaluated or produced an explainable effect.".to_string()
    );

    let effect = PropagatingEffect::Numerical(0.99);
    let eval = causaloid.evaluate(&effect);
    assert!(eval.is_ok());
    assert_eq!(eval.unwrap(), PropagatingEffect::Deterministic(true));

    let actual = causaloid.explain();
    assert!(actual.is_ok());
}

#[test]
fn test_causal_graph() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let (causal_graph, _) = test_utils_graph::get_small_multi_layer_cause_graph_and_data();

    let causaloid = Causaloid::from_causal_graph(id, Arc::new(causal_graph), description);
    assert!(!causaloid.is_singleton());

    assert!(causaloid.causal_graph().is_some());
    assert!(causaloid.causal_collection().is_none());
}
