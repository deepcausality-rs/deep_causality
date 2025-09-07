/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{BaseContext, Causable, CausalState, Identifiable, PropagatingEffect};

use deep_causality::utils_test::test_utils;
use std::sync::Arc;

#[test]
fn test_new() {
    let causaloid = test_utils::get_test_causaloid_deterministic();
    assert!(causaloid.is_singleton());
    assert_eq!(1, causaloid.id());
    assert_eq!(
        "tests whether data exceeds threshold of 0.55".to_string(),
        causaloid.description()
    );

    let id = 42;
    let version = 1;
    // CausalState now takes an PropagatingEffect enum.
    let data = PropagatingEffect::Numerical(0.23f64);
    let cs = CausalState::new(id, version, data, causaloid);

    assert_eq!(cs.id(), id);
    assert_eq!(cs.version(), version);
}

#[test]
fn test_eval() {
    let id = 42;
    let version = 1;
    let causaloid = test_utils::get_test_causaloid_deterministic();

    // Case 1: Evaluation results in Deterministic(false)
    let data_fail = PropagatingEffect::Numerical(0.23f64);
    let cs1 = CausalState::new(id, version, data_fail, causaloid.clone());

    let res = cs1.eval();
    assert!(res.is_ok());
    // The result is a PropagatingEffect, not a bool.
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(false));

    // Case 2: Evaluation results in Deterministic(true)
    let data_success = PropagatingEffect::Numerical(0.93f64);
    let cs2 = CausalState::new(id, version, data_success, causaloid);

    let res = cs2.eval();
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
}

#[test]
fn eval_with_data() {
    let id = 42;
    let version = 1;
    // The initial data in the state is often just a default.
    let initial_data = PropagatingEffect::None;
    let causaloid = test_utils::get_test_causaloid_deterministic();
    let cs = CausalState::new(id, version, initial_data, causaloid);

    // Evaluating with internal data (None) should fail the causaloid's check.
    let res = cs.eval();
    assert!(
        res.is_err(),
        "Evaluation with PropagatingEffect::None should fail"
    );

    // Now evaluate with external data.
    // Case 1: Fails evaluation
    let external_data_fail = PropagatingEffect::Numerical(0.11f64);
    let res = cs.eval_with_data(&external_data_fail);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(false));

    // Case 2: Succeeds evaluation
    let external_data_success = PropagatingEffect::Numerical(0.89f64);
    let res = cs.eval_with_data(&external_data_success);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), PropagatingEffect::Deterministic(true));
}

#[test]
fn test_to_string() {
    let causaloid = test_utils::get_test_causaloid_deterministic();
    assert!(causaloid.is_singleton());
    assert_eq!(1, causaloid.id());
    assert_eq!(
        "tests whether data exceeds threshold of 0.55".to_string(),
        causaloid.description()
    );

    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let cs = CausalState::new(id, version, data, causaloid);

    let expected =  "CausalState: id: 42 version: 1 data: {:?} causaloid: PropagatingEffect::Numerical(0.23) Causaloid id: 1 \n Causaloid type: Singleton \n description: tests whether data exceeds threshold of 0.55".to_string();
    let actual = cs.to_string();
    assert_eq!(actual, expected);
}

#[test]
fn test_context_getter() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);

    // Case 1: Causaloid has a context.
    let context = BaseContext::with_capacity(101, "test_context", 1);
    let context_arc = Arc::new(context.clone());
    let causaloid_with_context = test_utils::get_test_causaloid_deterministic_with_context(context);

    let cs_with_context = CausalState::new(id, version, data.clone(), causaloid_with_context);

    let retrieved_context_opt = cs_with_context.context();
    assert!(retrieved_context_opt.is_some());
    let retrieved_context = retrieved_context_opt.as_ref().unwrap();
    assert_eq!(retrieved_context.id(), context_arc.id());
    assert_eq!(retrieved_context.name(), context_arc.name());

    // Case 2: Causaloid has no context.
    let causaloid_no_context = test_utils::get_test_causaloid_deterministic();
    let cs_no_context = CausalState::new(id, version, data, causaloid_no_context);

    assert!(cs_no_context.context().is_none());
}
