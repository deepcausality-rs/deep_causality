/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils;
use deep_causality::utils_test::test_utils::get_base_context;
use deep_causality::{
    AggregateLogic, BaseCausaloid, Causable, Causaloid, EffectValue, IdentificationValue,
    MonadicCausableCollection, PropagatingEffect,
};
use std::sync::{Arc, RwLock};

#[test]
fn test_from_causal_collection() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let causal_coll_base = test_utils::get_deterministic_test_causality_vec();
    let causal_coll_ids = causal_coll_base.into_iter().collect();

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
    let causal_coll = Arc::new(causal_coll_base.into_iter().collect());
    let context = Arc::new(RwLock::new(get_base_context()));

    let causaloid: BaseCausaloid<f64, bool> = Causaloid::from_causal_collection_with_context(
        id,
        causal_coll,
        context,
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
    let causal_coll = Arc::new(vec![
        test_utils::get_test_causaloid_deterministic_true(),
        test_utils::get_test_causaloid_deterministic_false(),
        test_utils::get_test_causaloid_deterministic_true(),
    ]);

    let causaloid: BaseCausaloid<bool, bool> = BaseCausaloid::from_causal_collection(
        id,
        causal_coll.clone(),
        description,
        AggregateLogic::Any,
        0.5,
    );
    assert!(!causaloid.is_singleton());
    assert!(causaloid.causal_collection().is_some());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_none());

    // Evaluate the collection-based causaloid.
    // NOTE: Causaloid::evaluate does not support Collections due to generic bounds.
    // We must use evaluate_collection on the collection itself.
    let effect = PropagatingEffect::from_value(true);

    // We access the collection directly (using the Arc we created or from the getter)
    let res = causal_coll.evaluate_collection(&effect, &AggregateLogic::Any, Some(0.5));
    dbg!(&res);

    // The default aggregation for a collection is "true".
    assert_eq!(res.value, EffectValue::Value(true));
}

#[test]
fn test_explain_collection_success() {
    let causal_coll = Arc::new(vec![
        test_utils::get_test_causaloid_deterministic_true(),
        test_utils::get_test_causaloid_deterministic_false(),
    ]);

    // Setup: A collection causaloid
    let _collection_causaloid: BaseCausaloid<bool, bool> = Causaloid::from_causal_collection(
        104,
        causal_coll.clone(),
        "Explainable Collection",
        AggregateLogic::Any,
        0.5,
    );

    // Act: Evaluate the collection. Now both members will be evaluated.
    let effect = PropagatingEffect::from_value(false); // Changed to bool
    let res = causal_coll.evaluate_collection(&effect, &AggregateLogic::Any, Some(0.5));
    dbg!(&res);

    // Now, call explain.
    let explanation = res.explain(); // Original, correct

    // Assert: The explanation should contain the results from both sub-causaloids.
    assert!(explanation.contains("Outgoing effect: Value(true)"));
    assert!(explanation.contains("Outgoing effect: Value(false)"));
}

#[test]
fn test_evaluate_collection_with_sub_evaluation_error() {
    // Setup: A collection containing a causaloid that will return an error.
    let causal_coll = Arc::new(vec![
        test_utils::get_test_error_causaloid(),
        test_utils::get_test_causaloid_deterministic_true(),
    ]);

    let _collection_causaloid: BaseCausaloid<bool, bool> = Causaloid::from_causal_collection(
        102,
        causal_coll.clone(),
        "Error Collection",
        AggregateLogic::Any,
        0.5,
    );

    // Act
    let effect = PropagatingEffect::from_value(false); // Changed to bool
    let res = causal_coll.evaluate_collection(&effect, &AggregateLogic::Any, Some(0.5));
    dbg!(&res);

    // Assert: The error from the sub-causaloid should now be propagated up.
    assert!(res.error.is_some());
    let err = res.error.unwrap();
    assert!(err.to_string().contains("Test error"));
}

#[test]
fn test_evaluate_collection_without_true_effect() {
    // Setup: A collection with only 'false' causaloids.
    let causal_coll = Arc::new(vec![
        test_utils::get_test_causaloid_deterministic_false(),
        test_utils::get_test_causaloid_deterministic_false(),
    ]);

    let _collection_causaloid: BaseCausaloid<bool, bool> = Causaloid::from_causal_collection(
        101,
        causal_coll.clone(),
        "All False Collection",
        AggregateLogic::Any,
        0.5,
    );

    // Act
    let effect = PropagatingEffect::from_value(false); // Changed to bool
    let res = causal_coll.evaluate_collection(&effect, &AggregateLogic::Any, Some(0.5));
    dbg!(&res);

    // Assert: Since no causaloid is true, the aggregated effect should be false.
    assert_eq!(res.value, EffectValue::Value(false));
}
