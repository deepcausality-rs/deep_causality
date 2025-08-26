/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::csm_types::csm::csm_utils_test;
use deep_causality::utils_test::test_utils;
use deep_causality::{CSM, CausalState, PropagatingEffect};

#[test]
fn eval_all_states() {
    let id = 42;
    let version = 1;
    let data = test_utils::get_test_single_data(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = csm_utils_test::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let res = csm.eval_all_states();
    assert!(res.is_ok())
}

#[test]
fn update_all_states() {
    let id = 42;
    let version = 1;
    let data = test_utils::get_test_single_data(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data, causaloid.clone());
    let ca = csm_utils_test::get_test_action();

    let state_actions = &[(&cs, &ca)];

    let csm = CSM::new(state_actions, None);

    assert_eq!(csm.len(), 1);
    let data = test_utils::get_test_single_data(0.23f64);
    let cs2 = CausalState::new(2, 2, data, causaloid);
    let ca2 = csm_utils_test::get_test_action();

    let state_actions = &[(&cs, &ca), (&cs2, &ca2)];

    let res = csm.update_all_states(state_actions);
    assert!(res.is_ok());

    assert_eq!(csm.len(), 2)
}

// I've renamed the original test to be more descriptive.
// It correctly tests the success path where a state is inactive.
#[test]
fn eval_all_states_success_inactive_state() {
    let id = 42;
    let version = 1;
    // Data that makes the state inactive (0.23 < 0.55 threshold)
    let data = test_utils::get_test_single_data(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = csm_utils_test::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);
    let res = csm.eval_all_states();
    assert!(res.is_ok())
}

// New test for the success path where a state is active and the action fires.
#[test]
fn eval_all_states_success_active_state_fires_action() {
    let id = 42;
    let version = 1;
    // Data that makes the state active (0.6 > 0.55 threshold)
    let data = PropagatingEffect::Numerical(0.60f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(); // Returns Deterministic(true)

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = csm_utils_test::get_test_action(); // Succeeds
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let res = csm.eval_all_states();
    assert!(res.is_ok());
}

// New test for the first error branch: state evaluation fails.
#[test]
fn eval_all_states_error_eval_fails() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    // This causaloid's causal_fn always returns an error.
    let causaloid = csm_utils_test::get_test_error_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = csm_utils_test::get_test_action(); // Action won't be reached
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let res = csm.eval_all_states();

    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("CSM Causal Error: CausalityError: Error"));
}

// New test for the second error branch: action firing fails.
#[test]
fn eval_all_states_error_action_fails() {
    let id = 42;
    let version = 1;
    // Use data that makes the state active (0.60 > 0.55 threshold)
    let data = PropagatingEffect::Numerical(0.60f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(); // Returns Deterministic(true)

    let cs = CausalState::new(id, version, data, causaloid);
    // Use an action that is designed to fail.
    let ca = csm_utils_test::get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let res = csm.eval_all_states();

    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("CSM Action Error: ActionError: Error"));
}

// New test for the third error branch: non-deterministic effect.
#[test]
fn eval_all_states_error_non_deterministic() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = csm_utils_test::get_test_probabilistic_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = csm_utils_test::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let res = csm.eval_all_states();
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("Invalid non-deterministic effect"));
    assert!(err_msg.contains(&format!("for state {}", id)));
}
