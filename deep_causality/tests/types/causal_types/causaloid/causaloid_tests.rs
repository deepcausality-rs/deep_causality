/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use std::sync::Arc;

use deep_causality::utils_test::test_utils::get_base_context;
use deep_causality::utils_test::*;

// Helper function to unpack numerical evidence, used in test causal functions.
fn unpack_evidence(evidence: &Evidence) -> Result<NumericalValue, CausalityError> {
    if let Evidence::Numerical(val) = evidence {
        Ok(*val)
    } else {
        Err(CausalityError(format!(
            "Expected Numerical evidence, got: {evidence:?}"
        )))
    }
}

#[test]
fn test_new() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";

    // CausalFn now takes &Evidence and returns Result<PropagatingEffect, CausalityError>
    fn causal_fn(evidence: &Evidence) -> Result<PropagatingEffect, CausalityError> {
        let obs = unpack_evidence(evidence)?;
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

    // ContextualCausalFn now takes &Evidence and returns Result<PropagatingEffect, CausalityError>
    fn contextual_causal_fn(
        evidence: &Evidence,
        ctx: &Arc<BaseContext>,
    ) -> Result<PropagatingEffect, CausalityError> {
        let obs = unpack_evidence(evidence)?;
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
    let evidence = Evidence::Numerical(0.99);
    let effect = causaloid.evaluate(&evidence).unwrap();

    // The default aggregation for a collection is "any true".
    assert_eq!(effect, PropagatingEffect::Deterministic(true));
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
    let evidence = Evidence::Numerical(0.99);
    let res = causaloid.evaluate(&evidence);
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

    let evidence = Evidence::Numerical(0.99);
    let res = causaloid.evaluate(&evidence);
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

    let evidence = Evidence::Numerical(0.99);
    let eval = causaloid.evaluate(&evidence);
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

    let evidence = Evidence::Numerical(0.78);
    let res = causaloid.evaluate(&evidence).unwrap();
    assert_eq!(res, PropagatingEffect::Deterministic(true));

    let actual = causaloid.explain().unwrap();
    let expected = "Causaloid: 1 'tests whether data exceeds threshold of 0.55' evaluated to: Deterministic(true)".to_string();
    assert_eq!(actual, expected);
}

#[test]
fn test_evaluate_singleton() {
    let causaloid = test_utils::get_test_causaloid();

    let evidence = Evidence::Numerical(0.78);
    let res = causaloid.evaluate(&evidence).unwrap();
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
    let evidence = Evidence::Numerical(0.99);
    causaloid.evaluate(&evidence).unwrap();
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
    let evidence = Evidence::Numerical(0.99);
    causaloid.evaluate(&evidence).unwrap();
    let expected_active = "Causaloid id: 1 \n Causaloid type: Singleton \n description: tests whether data exceeds threshold of 0.55";
    let actual_active = format!("{causaloid:?}");
    assert_eq!(actual_active, expected_active);
}
