/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use deep_causality_core::CausalityErrorEnum;
use deep_causality_haft::MonadEffect5;
use std::sync::{Arc, RwLock};

use deep_causality::utils_test::test_utils::get_base_context;
use deep_causality::utils_test::*;
use deep_causality_haft::LogAddEntry;

#[test]
fn test_new() {
    let id: IdentificationValue = 1;
    let description = "tests whether data exceeds threshold of 0.55";

    fn causal_fn(obs: NumericalValue) -> PropagatingEffect<bool> {
        if obs.is_nan() {
            return PropagatingEffect::from_error(CausalityError::new(CausalityErrorEnum::Custom(
                "Observation is NAN".into(),
            )));
        }

        let threshold: NumericalValue = 0.75;
        let is_active = obs.ge(&threshold);
        let mut log = EffectLog::new();
        log.add_entry("Causal function executed successfully");

        let mut effect = CausalMonad::pure(is_active);
        effect.logs = log;
        effect
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
        obs: EffectValue<NumericalValue>,
        _state: (),
        ctx: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        let val = obs.into_value();
        if val.is_none() {
            // In PropagatingProcess, we return error wrapped in the struct
            return PropagatingProcess::from_error(CausalityError::new(
                CausalityErrorEnum::Custom("Observation is NULL".into()),
            ));
        }
        let obs_val = val.unwrap();
        if obs_val.is_nan() {
            return PropagatingProcess::from_error(CausalityError::new(
                CausalityErrorEnum::Custom("Observation is NULL/NAN".into()),
            ));
        }

        if ctx.is_none() {
            return PropagatingProcess::from_error(CausalityError::new(
                CausalityErrorEnum::Custom("Context is missing".into()),
            ));
        }

        // get context lock:
        let ctx_arc = ctx.unwrap();
        let ctx_lock = ctx_arc.read().unwrap();

        // get contextoid by ID
        // Note: get_base_context adds root node with ID 1 at index 0.
        // But ctx.get_node(index) gets by index.
        let contextoid = ctx_lock.get_node(0).expect("Could not find contextoid");

        // extract data from the contextoid
        let val = contextoid.id() as f64;

        // relate the observation (obs) to the data (val) from the contextoid
        let is_active = obs_val.ge(&val);

        let mut log = EffectLog::new();
        log.add_entry("Contextual causal function executed successfully");

        let mut process = PropagatingProcess::pure(is_active);
        process.logs = log;
        process
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

    let effect = PropagatingEffect::from_value(0.78);
    let res = causaloid.evaluate(&effect);
    assert!(res.is_ok());

    let actual = res.value;
    let expected = EffectValue::Value(true);
    assert_eq!(actual, expected);
}

#[test]
fn test_evaluate_singleton_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests a causaloid with a context";
    let context = get_base_context();

    fn contextual_causal_fn(
        obs: EffectValue<NumericalValue>,
        _state: (),
        ctx: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        let val = obs.into_value();
        // Error handling omitted for brevity in this specific test logic, similar to before
        let obs_val = val.unwrap();

        // get context lock:
        let ctx_arc = ctx.unwrap();
        let ctx_lock = ctx_arc.read().unwrap();
        // Get contextoid by ID. In get_base_context, the node at index 0 has ID 1.
        let contextoid = ctx_lock.get_node(0).expect("Could not find contextoid");
        // Extract a value from the contextoid.
        let val = contextoid.id() as f64; // This will be 1.0
        // Relate the observation (obs) to the data (val) from the contextoid.
        let is_active = obs_val.ge(&val);

        let mut log = EffectLog::new();
        log.add_entry("Contextual causal function executed successfully");

        let mut process = PropagatingProcess::pure(is_active);
        process.logs = log;
        process
    }

    let causaloid = BaseCausaloid::<NumericalValue, bool>::new_with_context(
        id,
        contextual_causal_fn,
        Arc::new(RwLock::new(context)),
        description,
    );

    // Evaluate with evidence that should result in true (1.5 >= 1.0)
    let effect_true = PropagatingEffect::from_value(1.5);
    let res_true = causaloid.evaluate(&effect_true);
    assert_eq!(res_true.value, EffectValue::Value(true));

    // Evaluate with evidence that should result in false (0.5 < 1.0)
    let effect_false = PropagatingEffect::from_value(0.5);
    let res_false = causaloid.evaluate(&effect_false);
    assert_eq!(res_false.value, EffectValue::Value(false));
}

// Removed test_evaluate_singleton_err as it was testing compile-time type mismatch with runtime logic.
