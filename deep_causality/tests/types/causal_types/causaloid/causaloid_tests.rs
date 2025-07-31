/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use std::sync::Arc;

use deep_causality::utils_test::test_utils::get_base_context;
use deep_causality::utils_test::*;

// Helper function to unpack numerical evidence, used in test causal functions.
fn unpack_evidence(effect: &PropagatingEffect) -> Result<NumericalValue, CausalityError> {
    if let PropagatingEffect::Numerical(val) = effect {
        Ok(*val)
    } else {
        Err(CausalityError(format!(
            "Expected Numerical effect, got: {effect:?}"
        )))
    }
}

#[test]
fn test_new() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";

    // CausalFn now takes &PropagatingEffect and returns Result<PropagatingEffect, CausalityError>
    fn causal_fn(effect: &PropagatingEffect) -> Result<PropagatingEffect, CausalityError> {
        let obs = unpack_evidence(effect)?;
        if obs.is_nan() {
            return Err(CausalityError("Observation is NULL/NAN".into()));
        }
        let threshold: NumericalValue = 0.75;
        let is_active = obs.ge(&threshold);
        Ok(PropagatingEffect::Deterministic(is_active))
    }

    let causaloid: BaseCausaloid = Causaloid::new(id, causal_fn, description);

    assert!(causaloid.is_singleton());
    assert!(causaloid.causal_collection().is_none());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_none());
}

#[test]
fn test_new_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let context = get_base_context();

    // ContextualCausalFn now takes &PropagatingEffect and returns Result<PropagatingEffect, CausalityError>
    fn contextual_causal_fn(
        effect: &PropagatingEffect,
        ctx: &Arc<BaseContext>,
    ) -> Result<PropagatingEffect, CausalityError> {
        let obs = unpack_evidence(effect)?;
        if obs.is_nan() {
            return Err(CausalityError("Observation is NULL/NAN".into()));
        }

        // get contextoid by ID
        let contextoid = ctx.get_node(0).expect("Could not find contextoid");

        // extract data from the contextoid
        let val = contextoid.id() as f64;

        // run any arithmetic with the data from the contextoid
        let is_active = if val == 1.0 {
            true
        } else {
            // relate the observation (obs) to the data (val) from the contextoid
            obs.ge(&val)
        };
        Ok(PropagatingEffect::Deterministic(is_active))
    }

    let causaloid: BaseCausaloid =
        Causaloid::new_with_context(id, contextual_causal_fn, Arc::new(context), description);

    assert!(causaloid.is_singleton());
    assert!(causaloid.causal_collection().is_none());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_some());
}

#[test]
fn test_collection_causaloid_evaluation() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let causal_coll = test_utils::get_test_causality_vec();

    let causaloid = Causaloid::from_causal_collection(id, Arc::new(causal_coll), description);
    assert!(!causaloid.is_singleton());

    // Evaluate the collection-based causaloid.
    let effect = PropagatingEffect::Numerical(0.99);
    let res = causaloid.evaluate(&effect).unwrap();

    // The default aggregation for a collection is "any true".
    assert_eq!(res, PropagatingEffect::Deterministic(true));
    assert!(causaloid.causal_collection().is_some());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_none());
}

#[test]
fn test_from_causal_collection() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let causal_coll = test_utils::get_test_causality_vec();

    let causaloid = Causaloid::from_causal_collection(id, Arc::new(causal_coll), description);
    assert!(!causaloid.is_singleton());

    assert!(causaloid.causal_collection().is_some());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_none());
}

#[test]
fn test_from_causal_collection_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let causal_coll = test_utils::get_test_causality_vec();
    let context = get_base_context();

    let causaloid = Causaloid::from_causal_collection_with_context(
        id,
        Arc::new(causal_coll),
        Arc::new(context),
        description,
    );

    assert!(!causaloid.is_singleton());
    assert!(causaloid.causal_collection().is_some());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_some());
}

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
        Arc::new(context),
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
fn test_causal_graph() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";
    let (causal_graph, _) = test_utils_graph::get_small_multi_layer_cause_graph_and_data();

    let causaloid = Causaloid::from_causal_graph(id, Arc::new(causal_graph), description);
    assert!(!causaloid.is_singleton());

    assert!(causaloid.causal_graph().is_some());
    assert!(causaloid.causal_collection().is_none());
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
fn test_explain() {
    let causaloid = test_utils::get_test_causaloid();
    // Before evaluation, state is unknown.
    assert!(causaloid.explain().is_err());

    let effect = PropagatingEffect::Numerical(0.78);
    let res = causaloid.evaluate(&effect).unwrap();
    assert_eq!(res, PropagatingEffect::Deterministic(true));

    let actual = causaloid.explain().unwrap();
    let expected = "Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: PropagatingEffect::Deterministic(true)".to_string();
    assert_eq!(actual, expected);
}

#[test]
fn test_evaluate_singleton() {
    let causaloid = test_utils::get_test_causaloid();

    let effect = PropagatingEffect::Numerical(0.78);
    let res = causaloid.evaluate(&effect).unwrap();
    assert_eq!(res, PropagatingEffect::Deterministic(true));
}

#[test]
fn test_to_string() {
    let causaloid = test_utils::get_test_causaloid();
    // Before evaluation, is_active returns an error, which the Display trait should handle.
    let expected_unevaluated = "Causaloid id: 1 \n Causaloid type: Singleton \n description: tests whether data exceeds threshold of 0.55";
    let actual_unevaluated = causaloid.to_string();
    assert_eq!(actual_unevaluated, expected_unevaluated);

    // Evaluate to active
    let effect = PropagatingEffect::Numerical(0.99);
    causaloid.evaluate(&effect).unwrap();
    let expected_active = "Causaloid id: 1 \n Causaloid type: Singleton \n description: tests whether data exceeds threshold of 0.55";
    let actual_active = causaloid.to_string();
    assert_eq!(actual_active, expected_active);
}

#[test]
fn test_debug() {
    let causaloid = test_utils::get_test_causaloid();
    // Before evaluation, is_active returns an error, which the Debug trait should handle.
    let expected_unevaluated = "Causaloid id: 1 \n Causaloid type: Singleton \n description: tests whether data exceeds threshold of 0.55";
    let actual_unevaluated = format!("{causaloid:?}");
    assert_eq!(actual_unevaluated, expected_unevaluated);

    // Evaluate to active
    let effect = PropagatingEffect::Numerical(0.99);
    causaloid.evaluate(&effect).unwrap();
    let expected_active = "Causaloid id: 1 \n Causaloid type: Singleton \n description: tests whether data exceeds threshold of 0.55";
    let actual_active = format!("{causaloid:?}");
    assert_eq!(actual_active, expected_active);
}

#[test]
fn test_evaluate_collection_without_true_effect() {
    // Setup: A collection with only 'false' causaloids.
    let false_causaloid1 = test_utils::get_test_causaloid_deterministic_false();
    let false_causaloid2 = test_utils::get_test_causaloid_deterministic_false();
    let causal_coll = vec![false_causaloid1, false_causaloid2];
    let collection_causaloid =
        Causaloid::from_causal_collection(101, Arc::new(causal_coll), "All False Collection");

    // Act
    let effect = PropagatingEffect::Numerical(0.0);
    let res = collection_causaloid.evaluate(&effect).unwrap();

    // Assert: Since no causaloid is true, the aggregated effect should be false.
    assert_eq!(res, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_evaluate_collection_with_sub_evaluation_error() {
    // Setup: A collection containing a causaloid that will return an error.
    let error_causaloid = test_utils::get_test_error_causaloid();
    let true_causaloid = test_utils::get_test_causaloid_deterministic_true();

    // The error_causaloid must come first to ensure it gets evaluated.
    let causal_coll = vec![error_causaloid, true_causaloid]; // <-- The order is swapped here.

    let collection_causaloid =
        Causaloid::from_causal_collection(102, Arc::new(causal_coll), "Error Collection");

    // Act
    let effect = PropagatingEffect::Numerical(0.0);
    let result = collection_causaloid.evaluate(&effect);

    // Assert: The error from the sub-causaloid should now be propagated up.
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Test error"));
}
#[test]
fn test_explain_collection_success() {
    // Setup: A collection causaloid that has been evaluated.
    let true_causaloid = test_utils::get_test_causaloid_deterministic_true();
    let false_causaloid = test_utils::get_test_causaloid_deterministic_false();

    // The `false` causaloid must come first to ensure the `evaluate` loop
    // does not short-circuit before evaluating both.
    let causal_coll = vec![false_causaloid, true_causaloid]; // <-- Swapped order

    let collection_causaloid =
        Causaloid::from_causal_collection(104, Arc::new(causal_coll), "Explainable Collection");

    // Act: Evaluate the collection. Now both members will be evaluated.
    let effect = PropagatingEffect::Numerical(0.0);
    collection_causaloid.evaluate(&effect).unwrap();

    // Now, call explain.
    let explanation = collection_causaloid.explain().unwrap();

    // Assert: The explanation should contain the results from both sub-causaloids.
    assert!(explanation.contains("evaluated to: PropagatingEffect::Deterministic(true)"));
    assert!(explanation.contains("evaluated to: PropagatingEffect::Deterministic(false)"));
}
// This test covers an error path in explain() for a Collection Causaloid.
#[test]
fn test_explain_collection_with_sub_explain_error() {
    // Setup: A collection where one causaloid will not be evaluated due to short-circuiting.
    let true_causaloid = test_utils::get_test_causaloid_deterministic_true();
    let unevaluated_causaloid = test_utils::get_test_causaloid(); // This one will remain unevaluated.

    let causal_coll = vec![true_causaloid, unevaluated_causaloid];
    let collection_causaloid = Causaloid::from_causal_collection(
        105,
        Arc::new(causal_coll),
        "Sub-explain Error Collection",
    );

    // Act: Evaluate the collection. The evaluation will stop after the first `true` effect.
    let effect = PropagatingEffect::Numerical(0.0);
    collection_causaloid.evaluate(&effect).unwrap();

    // Now, call explain. This will fail because the second causaloid was never evaluated.
    let result = collection_causaloid.explain();

    // Assert: The result should be an error.
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("has not been evaluated"));
}

#[test]
fn test_evaluate_singleton_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests a causaloid with a context";
    let context = get_base_context();

    fn contextual_causal_fn(
        effect: &PropagatingEffect,
        ctx: &Arc<BaseContext>,
    ) -> Result<PropagatingEffect, CausalityError> {
        let obs = unpack_evidence(effect)?;
        // Get contextoid by ID. In get_base_context, the node at index 0 has ID 1.
        let contextoid = ctx.get_node(0).expect("Could not find contextoid");
        // Extract a value from the contextoid.
        let val = contextoid.id() as f64; // This will be 1.0
        // Relate the observation (obs) to the data (val) from the contextoid.
        let is_active = obs.ge(&val);
        Ok(PropagatingEffect::Deterministic(is_active))
    }

    let causaloid: BaseCausaloid =
        Causaloid::new_with_context(id, contextual_causal_fn, Arc::new(context), description);

    // Evaluate with evidence that should result in true (1.5 >= 1.0)
    let effect_true = PropagatingEffect::Numerical(1.5);
    let res_true = causaloid.evaluate(&effect_true).unwrap();
    assert_eq!(res_true, PropagatingEffect::Deterministic(true));

    // Evaluate with evidence that should result in false (0.5 < 1.0)
    let effect_false = PropagatingEffect::Numerical(0.5);
    let res_false = causaloid.evaluate(&effect_false).unwrap();
    assert_eq!(res_false, PropagatingEffect::Deterministic(false));
}

#[test]
fn test_evaluate_collection_ignores_other_effects() {
    // Setup: A collection with causaloids that have effects other than Deterministic or Halting.
    let probabilistic_causaloid = test_utils::get_test_causaloid_probabilistic();
    let contextual_link_causaloid = test_utils::get_test_causaloid_contextual_link();
    let false_causaloid = test_utils::get_test_causaloid_deterministic_false();

    let causal_coll = vec![
        probabilistic_causaloid,
        contextual_link_causaloid,
        false_causaloid,
    ];
    let collection_causaloid =
        Causaloid::from_causal_collection(103, Arc::new(causal_coll), "Ignore Others Collection");

    // Act
    let effect = PropagatingEffect::Numerical(0.0);
    let res = collection_causaloid.evaluate(&effect).unwrap();

    // Assert: The aggregation logic should ignore Probabilistic and ContextualLink,
    // resulting in an overall effect of Deterministic(false).
    assert_eq!(res, PropagatingEffect::Deterministic(false));
}
