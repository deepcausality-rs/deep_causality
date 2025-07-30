/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{Causable, CausalState, Identifiable, PropagatingEffect};

use deep_causality::utils_test::test_utils;

#[test]
fn test_new() {
    let causaloid = test_utils::get_test_causaloid();
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

    assert_eq!(*cs.id(), id);
    assert_eq!(*cs.version(), version);
}

#[test]
fn test_eval() {
    let id = 42;
    let version = 1;
    let causaloid = test_utils::get_test_causaloid();

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
    let causaloid = test_utils::get_test_causaloid();
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
    let causaloid = test_utils::get_test_causaloid();
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

    // The expected string needs to be updated to match the new Debug format of PropagatingEffect
    // and the Display format of an unevaluated Causaloid.
    let expected = "CausalState: \n id: 42 version: 1 \n data: PropagatingEffect::Numerical(0.23) causaloid: Causaloid id: 1 \n Causaloid type: Singleton \n description: tests whether data exceeds threshold of 0.55".to_string();
    let actual = cs.to_string();
    assert_eq!(actual, expected);
}
