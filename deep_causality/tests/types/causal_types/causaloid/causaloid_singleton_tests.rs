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

    fn causal_fn(obs: NumericalValue) -> Result<CausalFnOutput<bool>, CausalityError> {
        if obs.is_nan() {
            return Err(CausalityError("Observation is NULL/NAN".into()));
        }
        let threshold: NumericalValue = 0.75;
        let is_active = obs.ge(&threshold);
        let mut log = CausalEffectLog::new();
        log.add_entry("Causal function executed successfully");

        Ok(CausalFnOutput::new(is_active, log))
    }

    let causaloid = BaseCausaloid::<NumericalValue, bool>::new(id, causal_fn, description);

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

    fn contextual_causal_fn(
        obs: NumericalValue,
        ctx: &Arc<RwLock<BaseContext>>,
    ) -> Result<CausalFnOutput<bool>, CausalityError> {
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

        let mut log = CausalEffectLog::new();
        log.add_entry("Contextual causal function executed successfully");

        Ok(CausalFnOutput::new(is_active, log))
    }

    let causaloid = BaseCausaloid::<NumericalValue, bool>::new_with_context(
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
    let causaloid = test_utils::get_test_causaloid_deterministic(24);
    let expected = format!(
        "Causaloid id: {} \n Causaloid type: Singleton \n description: {}",
        causaloid.id(),
        causaloid.description()
    );
    let actual = causaloid.to_string();
    assert_eq!(actual, expected);
}

#[test]
fn test_evaluate_singleton() {
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let effect = PropagatingEffect::from_numerical(0.78);
    let res = causaloid.evaluate(&effect);
    assert!(res.is_ok());

    let actual = res.value;
    let expected = EffectValue::Deterministic(true);
    assert_eq!(actual, expected);
}

#[test]
fn test_evaluate_singleton_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests a causaloid with a context";
    let context = get_base_context();

    fn contextual_causal_fn(
        obs: NumericalValue,
        ctx: &Arc<RwLock<BaseContext>>,
    ) -> Result<CausalFnOutput<bool>, CausalityError> {
        // get context lock:
        let ctx = ctx.read().unwrap();
        // Get contextoid by ID. In get_base_context, the node at index 0 has ID 1.
        let contextoid = ctx.get_node(0).expect("Could not find contextoid");
        // Extract a value from the contextoid.
        let val = contextoid.id() as f64; // This will be 1.0
        // Relate the observation (obs) to the data (val) from the contextoid.
        let is_active = obs.ge(&val);

        let mut log = CausalEffectLog::new();
        log.add_entry("Contextual causal function executed successfully");
        Ok(CausalFnOutput::new(is_active, log))
    }

    let causaloid = BaseCausaloid::<NumericalValue, bool>::new_with_context(
        id,
        contextual_causal_fn,
        Arc::new(RwLock::new(context)),
        description,
    );

    // Evaluate with evidence that should result in true (1.5 >= 1.0)
    let effect_true = PropagatingEffect::from_numerical(1.5);
    let res_true = causaloid.evaluate(&effect_true);
    assert_eq!(res_true.value, EffectValue::Deterministic(true));

    // Evaluate with evidence that should result in false (0.5 < 1.0)
    let effect_false = PropagatingEffect::from_numerical(0.5);
    let res_false = causaloid.evaluate(&effect_false);
    assert_eq!(res_false.value, EffectValue::Deterministic(false));
}

#[test]
fn test_evaluate_singleton_err() {
    let causaloid: BaseCausaloid<bool, bool> = get_test_causaloid_deterministic_input_output();

    // The causal function expects a Boolean effect, but we pass in a Numerical effect.
    let effect = PropagatingEffect::from_numerical(0.5);
    // The result should be an error.
    let res = causaloid.evaluate(&effect);
    assert!(res.is_err());
}
