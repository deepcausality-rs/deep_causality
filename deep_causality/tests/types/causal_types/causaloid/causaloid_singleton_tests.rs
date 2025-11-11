/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use std::sync::{Arc, RwLock};

use deep_causality::utils_test::test_utils::{
    get_base_context, get_test_causaloid_deterministic_input_output,
};
use deep_causality::utils_test::*;

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
        ctx: &Arc<RwLock<BaseContext>>,
    ) -> Result<PropagatingEffect, CausalityError> {
        let obs = unpack_evidence(effect)?;
        if obs.is_nan() {
            return Err(CausalityError("Observation is NULL/NAN".into()));
        }

        // get context lock:
        let ctx = ctx.read().unwrap();

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

    let causaloid: BaseCausaloid = Causaloid::new_with_context(
        id,
        contextual_causal_fn,
        Arc::new(RwLock::new(context)),
        description,
    );

    assert!(causaloid.is_singleton());
    assert!(causaloid.causal_collection().is_none());
    assert!(causaloid.causal_graph().is_none());
    assert!(causaloid.context().is_some());
}

#[test]
fn test_explain() {
    let causaloid = test_utils::get_test_causaloid_deterministic();
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
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let effect = PropagatingEffect::Numerical(0.78);
    let res = causaloid.evaluate(&effect).unwrap();
    assert_eq!(res, PropagatingEffect::Deterministic(true));
}

#[test]
fn test_evaluate_singleton_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests a causaloid with a context";
    let context = get_base_context();

    fn contextual_causal_fn(
        effect: &PropagatingEffect,
        ctx: &Arc<RwLock<BaseContext>>,
    ) -> Result<PropagatingEffect, CausalityError> {
        let obs = unpack_evidence(effect)?;
        // get context lock:
        let ctx = ctx.read().unwrap();
        // Get contextoid by ID. In get_base_context, the node at index 0 has ID 1.
        let contextoid = ctx.get_node(0).expect("Could not find contextoid");
        // Extract a value from the contextoid.
        let val = contextoid.id() as f64; // This will be 1.0
        // Relate the observation (obs) to the data (val) from the contextoid.
        let is_active = obs.ge(&val);
        Ok(PropagatingEffect::Deterministic(is_active))
    }

    let causaloid: BaseCausaloid = Causaloid::new_with_context(
        id,
        contextual_causal_fn,
        Arc::new(RwLock::new(context)),
        description,
    );

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
fn test_evaluate_singleton_err() {
    let causaloid: BaseCausaloid = get_test_causaloid_deterministic_input_output();

    // The causal function expects a Deterministic effect, but we pass in a Probabilistic effect.
    let effect = PropagatingEffect::Probabilistic(4.2);
    // The result should be an error.
    let res = causaloid.evaluate(&effect);
    assert!(res.is_err());
}
