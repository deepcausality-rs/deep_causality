// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{Causable, CausalState, Identifiable};

use crate::utils::test_utils;

#[test]
fn test_new() {
    let causaloid = &test_utils::get_test_causaloid();
    assert!(causaloid.is_singleton());
    assert_eq!(01, causaloid.id());
    assert_eq!(
        "tests whether data exceeds threshold of 0.55".to_string(),
        causaloid.description()
    );
    assert!(!causaloid.is_active());

    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let cs = CausalState::new(id, version, data, causaloid);

    assert_eq!(*cs.id(), id);
    assert_eq!(*cs.version(), version);
    assert_eq!(*cs.data(), data);
    assert_eq!(*cs.causaloid(), causaloid);
}

#[test]
fn test_eval() {
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = &test_utils::get_test_causaloid();

    let cs1 = CausalState::new(id, version, data, causaloid);

    let res = cs1.eval();
    assert!(res.is_ok());

    let trigger = res.expect("Failed to unwrap eval result from causal state");
    assert!(!trigger);

    let data = 0.93f64;
    let cs2 = CausalState::new(id, version, data, causaloid);

    let res = cs2.eval();
    assert!(res.is_ok());

    let trigger = res.expect("Failed to unwrap eval result from causal state");
    assert!(trigger);
}

#[test]
fn eval_with_data() {
    let id = 42;
    let version = 1;
    let data = 0.0f64;
    let causaloid = &test_utils::get_test_causaloid();
    let cs = CausalState::new(id, version, data, causaloid);

    let res = cs.eval();
    assert!(res.is_ok());

    let trigger = res.expect("Failed to unwrap eval result from causal state");
    assert!(!trigger);

    let data = &0.0f64;
    let res = cs.eval_with_data(data);
    assert!(res.is_ok());

    let trigger = res.expect("Failed to unwrap eval result from causal state");
    assert!(!trigger);

    let data = &0.89f64;
    let res = cs.eval_with_data(data);
    assert!(res.is_ok());

    let trigger = res.expect("Failed to unwrap eval result from causal state");
    assert!(trigger);
}

#[test]
fn test_to_string() {
    let causaloid = &test_utils::get_test_causaloid();
    assert!(causaloid.is_singleton());
    assert_eq!(01, causaloid.id());
    assert_eq!(
        "tests whether data exceeds threshold of 0.55".to_string(),
        causaloid.description()
    );
    assert!(!causaloid.is_active());

    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let cs = CausalState::new(id, version, data, causaloid);

    let expected = "CausalState: \n id: 42 version: 1 \n data: 0.23 causaloid: Causaloid id: 1 \n Causaloid type: Singleton \n description: tests whether data exceeds threshold of 0.55 is active: false has context: false".to_string();
    let actual = cs.to_string();
    assert_eq!(actual, expected)
}
