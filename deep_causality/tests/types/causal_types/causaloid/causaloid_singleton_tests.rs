/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;
use deep_causality_core::CausalityErrorEnum;
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

        PropagatingEffect::from_value_with_log(is_active, log)
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
        obs: CausalEffect<NumericalValue>,
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

        PropagatingProcess::from_value_with_log(is_active, log)
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

    let actual = res.value();
    let expected = Some(&true);
    assert_eq!(actual, expected);
}

#[test]
fn test_evaluate_singleton_errors_on_a_relay_command_input() {
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    // A command (`RelayTo`) on the input channel cannot be consumed by a singleton: it carries no
    // input value and cannot be retyped `I -> O`. The stateless path surfaces a clear,
    // command-specific error (mirroring `evaluate_stateful`) rather than reporting a generic
    // "input value is None", which would conflate a dropped command with absence of evidence.
    let effect: PropagatingEffect<NumericalValue> =
        PropagatingEffect::from_effect(CausalEffect::relay_to(5, CausalEffect::value(0.78)));
    let res = causaloid.evaluate(&effect);

    assert!(res.is_err());
    let err = res.error().expect("a command input errors");
    assert!(format!("{err:?}").contains("received a command"));
}

#[test]
fn test_evaluate_singleton_with_context() {
    let id: IdentificationValue = 1;
    let description = "tests a causaloid with a context";
    let context = get_base_context();

    fn contextual_causal_fn(
        obs: CausalEffect<NumericalValue>,
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

        PropagatingProcess::from_value_with_log(is_active, log)
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
    assert_eq!(res_true.value(), Some(&true));

    // Evaluate with evidence that should result in false (0.5 < 1.0)
    let effect_false = PropagatingEffect::from_value(0.5);
    let res_false = causaloid.evaluate(&effect_false);
    assert_eq!(res_false.value(), Some(&false));
}

// Removed test_evaluate_singleton_err as it was testing compile-time type mismatch with runtime logic.

#[test]
fn test_evaluate_collection_error() {
    fn causal_fn(_input: ()) -> PropagatingEffect<bool> {
        PropagatingEffect::from_value(true)
    }

    let c1 = BaseCausaloid::<(), bool>::new(1, causal_fn, "c1");
    let c_coll = BaseCausaloid::<(), bool>::from_causal_collection(
        3,
        Arc::new(vec![c1]),
        "collection",
        AggregateLogic::All,
        0.0,
    );

    let effect = PropagatingEffect::from_value(());
    let res = c_coll.evaluate(&effect);

    assert!(res.is_err());
    let err_msg = res.error().unwrap().to_string();
    assert!(err_msg.contains("Collection evaluation is not available"));
}

#[test]
fn test_evaluate_graph_error() {
    // Use existing test utility to build a valid causal graph
    let graph = test_utils_graph::build_multi_cause_graph();
    let c_graph: BaseCausaloid<f64, f64> =
        Causaloid::from_causal_graph(4, "graph from utility", Arc::new(graph));

    let effect = PropagatingEffect::from_value(1.0);
    let res = c_graph.evaluate(&effect);

    assert!(res.is_err());
    let err_msg = res.error().unwrap().to_string();
    assert!(err_msg.contains("Graph evaluation is not available"));
}

#[test]
fn test_contextual_fn_returning_none() {
    let id: IdentificationValue = 99;
    let description = "test none return";
    let context = get_base_context();

    fn bad_fn(
        _obs: CausalEffect<NumericalValue>,
        _state: (),
        _ctx: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        // Return a process with None value and None error
        PropagatingProcess::none()
    }

    let causaloid = BaseCausaloid::<NumericalValue, bool>::new_with_context(
        id,
        bad_fn,
        Arc::new(RwLock::new(context)),
        description,
    );

    let effect = PropagatingEffect::from_value(1.0);
    // This should trigger "context_fn returned None value and no error"
    let res = causaloid.evaluate(&effect);

    assert!(res.is_err());
    let err = res.error().unwrap().to_string();
    assert!(err.contains("context_fn returned None value"));
}

#[test]
fn test_errored_process_propagates_error_without_value() {
    // A carrier holding a value AND an error at once is unrepresentable now:
    // the outcome channel is value XOR error. An errored closure output must
    // propagate the error and carry no value.
    let id: IdentificationValue = 100;
    let description = "test error propagation";
    let context = get_base_context();

    fn problematic_fn(
        _obs: CausalEffect<f64>,
        _state: (),
        _ctx: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<f64, (), Arc<RwLock<BaseContext>>> {
        PropagatingProcess::from_error(CausalityError::new(CausalityErrorEnum::Custom(
            "This error should take priority".into(),
        )))
    }

    let causaloid = BaseCausaloid::<f64, f64>::new_with_context(
        id,
        problematic_fn,
        Arc::new(RwLock::new(context)),
        description,
    );

    let effect = PropagatingEffect::from_value(1.0);
    let result = causaloid.evaluate(&effect);

    assert!(result.is_err(), "Error must be propagated");
    assert!(
        result.value().is_none(),
        "An errored carrier must hold no value"
    );
    let err_msg = result.error().unwrap().to_string();
    assert!(err_msg.contains("This error should take priority"));
}

#[test]
fn test_relay_to_preservation() {
    let id: IdentificationValue = 102;
    let description = "test relay_to preservation";
    let context = get_base_context();

    fn relay_causal_fn(
        _obs: CausalEffect<NumericalValue>,
        _state: (),
        _ctx: Option<Arc<RwLock<BaseContext>>>,
    ) -> PropagatingProcess<bool, (), Arc<RwLock<BaseContext>>> {
        PropagatingProcess::from_effect(CausalEffect::relay_to(5, CausalEffect::value(true)))
    }

    let causaloid = BaseCausaloid::<NumericalValue, bool>::new_with_context(
        id,
        relay_causal_fn,
        Arc::new(RwLock::new(context)),
        description,
    );

    let effect = PropagatingEffect::from_value(0.5);
    let result = causaloid.evaluate(&effect);

    assert!(result.is_ok());
    assert!(result.command_target() == Some(5));
}

#[test]
fn test_none_output_error() {
    let id: IdentificationValue = 103;
    let description = "test none output error";

    fn none_fn(_obs: NumericalValue) -> PropagatingEffect<bool> {
        PropagatingEffect::from_effect(CausalEffect::none())
    }

    let causaloid = BaseCausaloid::<NumericalValue, bool>::new(id, none_fn, description);

    let effect = PropagatingEffect::from_value(0.5);
    let result = causaloid.evaluate(&effect);

    assert!(
        result.is_err(),
        "Result should be an error when causal function returns None"
    );
    let err_msg = result.error().unwrap().to_string();
    assert!(err_msg.contains("causal_fn returned None output"));
}
