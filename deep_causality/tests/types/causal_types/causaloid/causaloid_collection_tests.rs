/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils;
use deep_causality::utils_test::test_utils::{get_base_context, get_test_causaloid_deterministic};
use deep_causality::{
    AggregateLogic, BaseCausaloid, Causable, Causaloid, CausaloidId, CausaloidRegistry,
    EffectValue, Identifiable, IdentificationValue, MonadicCausable, PropagatingEffect,
};
use std::sync::{Arc, RwLock};

#[test]
fn test_from_causal_collection() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let causal_coll_base = test_utils::get_deterministic_test_causality_vec();
    let causal_coll_ids: Vec<CausaloidId> = causal_coll_base.iter().map(|c| c.id()).collect();

    let causaloid: BaseCausaloid<f64, bool> = BaseCausaloid::from_causal_collection(
        id,
        Arc::new(causal_coll_ids),
        description,
        AggregateLogic::Any,
        0.5,
    );
    assert!(!causaloid.is_singleton());

    assert!(causaloid.causal_collection().is_some());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_none());
}

#[test]
fn test_from_causal_collection_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let causal_coll_base = test_utils::get_deterministic_test_causality_vec();
    let causal_coll_ids: Vec<CausaloidId> = causal_coll_base.iter().map(|c| c.id()).collect();
    let context = get_base_context();

    let causaloid: BaseCausaloid<f64, bool> = Causaloid::from_causal_collection_with_context(
        id,
        Arc::new(causal_coll_ids),
        Arc::new(RwLock::new(context)),
        description,
        AggregateLogic::Any,
        0.5,
    );

    assert!(!causaloid.is_singleton());
    assert!(causaloid.causal_collection().is_some());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_some());
}

#[test]
fn test_collection_causaloid_evaluation() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let causal_coll_base = test_utils::get_deterministic_test_causality_vec();
    let causal_coll_ids: Vec<CausaloidId> = causal_coll_base.iter().map(|c| c.id()).collect();

    let mut registry = CausaloidRegistry::new();
    registry.register(get_test_causaloid_deterministic(1));

    let causaloid: BaseCausaloid<f64, bool> = BaseCausaloid::from_causal_collection(
        id,
        Arc::new(causal_coll_ids),
        description,
        AggregateLogic::Any,
        0.5,
    );
    assert!(!causaloid.is_singleton());

    // Evaluate the collection-based causaloid.
    let effect = PropagatingEffect::from_numerical(0.99);
    let res = causaloid.evaluate(&registry, &effect);
    dbg!(&res);

    // The default aggregation for a collection is "any true".
    assert_eq!(res.value, EffectValue::Deterministic(true));
    assert!(causaloid.causal_collection().is_some());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_none());
}

#[test]
fn test_explain_collection_success() {
    // Setup: A collection causaloid that has been evaluated.
    let true_causaloid = test_utils::get_test_causaloid_deterministic_true();
    let false_causaloid = test_utils::get_test_causaloid_deterministic_false();
    let mut registry = CausaloidRegistry::new();
    registry.register(true_causaloid.clone());
    registry.register(false_causaloid.clone());

    // The `false` causaloid must come first to ensure the `evaluate` loop
    // does not short-circuit before evaluating both.
    let causal_coll_base = [false_causaloid, true_causaloid]; // <-- Swapped order
    let causal_coll_ids: Vec<CausaloidId> = causal_coll_base.iter().map(|c| c.id()).collect();

    let collection_causaloid: BaseCausaloid<f64, bool> = Causaloid::from_causal_collection(
        104,
        Arc::new(causal_coll_ids),
        "Explainable Collection",
        AggregateLogic::Any,
        0.5,
    );

    // Act: Evaluate the collection. Now both members will be evaluated.
    let effect = PropagatingEffect::from_numerical(0.0);
    let res = collection_causaloid.evaluate(&registry, &effect);
    dbg!(&res);

    // Now, call explain.
    let explanation = res.explain(); // Original, correct

    // Assert: The explanation should contain the results from both sub-causaloids.
    assert!(explanation.contains("evaluated to: PropagatingEffect::Deterministic(true)"));
    assert!(explanation.contains("evaluated to: PropagatingEffect::Deterministic(false)"));
}

#[test]
fn test_evaluate_collection_with_sub_evaluation_error() {
    // Setup: A collection containing a causaloid that will return an error.
    let error_causaloid = test_utils::get_test_error_causaloid();
    let true_causaloid = test_utils::get_test_causaloid_deterministic_true();

    // The error_causaloid must come first to ensure it gets evaluated.
    let causal_coll_base = [error_causaloid, true_causaloid]; // <-- The order is swapped here.
    let causal_coll_ids: Vec<CausaloidId> = causal_coll_base.iter().map(|c| c.id()).collect();

    let collection_causaloid: BaseCausaloid<f64, bool> = Causaloid::from_causal_collection(
        102,
        Arc::new(causal_coll_ids),
        "Error Collection",
        AggregateLogic::Any,
        0.5,
    );

    // Act
    let registry = CausaloidRegistry::new();
    let effect = PropagatingEffect::from_numerical(0.0);
    let res = collection_causaloid.evaluate(&registry, &effect);
    dbg!(&res);

    // Assert: The error from the sub-causaloid should now be propagated up.
    assert!(res.error.is_some());
    let err = res.error.unwrap();
    assert!(err.to_string().contains("Test error"));
}

#[test]
fn test_evaluate_collection_without_true_effect() {
    // Setup: A collection with only 'false' causaloids.
    let false_causaloid1 = test_utils::get_test_causaloid_deterministic_false();
    let false_causaloid2 = test_utils::get_test_causaloid_deterministic_false();
    let mut registry = CausaloidRegistry::new();
    registry.register(false_causaloid1.clone());
    registry.register(false_causaloid2.clone());

    let causal_coll_base = [false_causaloid1, false_causaloid2];
    let causal_coll_ids: Vec<CausaloidId> = causal_coll_base.iter().map(|c| c.id()).collect();

    let collection_causaloid: BaseCausaloid<f64, bool> = Causaloid::from_causal_collection(
        101,
        Arc::new(causal_coll_ids),
        "All False Collection",
        AggregateLogic::Any,
        0.5,
    );

    // Act
    let effect = PropagatingEffect::from_numerical(0.0);
    let res = collection_causaloid.evaluate(&registry, &effect);
    dbg!(&res);

    // Assert: Since no causaloid is true, the aggregated effect should be false.
    assert_eq!(res.value, EffectValue::Deterministic(false));
}
