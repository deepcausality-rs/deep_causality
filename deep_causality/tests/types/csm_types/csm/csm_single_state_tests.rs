/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::utils_test::test_utils_csm;
use deep_causality::{CSM, CausalState, PropagatingEffect, UncertainParameter};
use deep_causality_uncertain::Uncertain;

#[test]
fn add_single_state() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data, causaloid.clone(), None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    assert_eq!(csm.len(), 1);
    let data = PropagatingEffect::Numerical(0.23f64);
    let cs2 = CausalState::new(2, 2, data, causaloid, None);
    let ca2 = test_utils_csm::get_test_action();
    let state_action = (cs2, ca2);

    let res = csm.add_single_state(43, state_action);

    assert!(res.is_ok());
    assert_eq!(csm.len(), 2);
}

#[test]
fn add_single_state_err_already_exists() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data, causaloid.clone(), None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    assert_eq!(csm.len(), 1);
    let data = PropagatingEffect::Numerical(0.23f64);
    let cs2 = CausalState::new(2, 2, data, causaloid, None);
    let ca2 = test_utils_csm::get_test_action();
    let state_action = (cs2, ca2);

    let res = csm.add_single_state(id, state_action);

    assert!(res.is_err());
    assert_eq!(csm.len(), 1);
}

#[test]
fn update_single_state() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();

    let state_action = &[(&cs, &ca)];

    let csm = CSM::new(state_action, None);
    assert_eq!(csm.len(), 1);

    let id = 44;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.7f64);

    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();

    let state_action = (cs, ca);

    let res = csm.update_single_state(42, state_action);
    assert!(res.is_ok());
    assert_eq!(csm.len(), 1);
}

#[test]
fn update_single_state_err_not_found() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let res = csm.update_single_state(99, (cs, ca));
    assert!(res.is_err());
}

#[test]
fn remove_single_state() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data.clone(), causaloid.clone(), None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    assert_eq!(csm.len(), 1);

    let cs2 = CausalState::new(2, 2, data, causaloid, None);
    let ca2 = test_utils_csm::get_test_action();
    let state_action = (cs2, ca2);

    let res = csm.add_single_state(43, state_action);

    assert!(res.is_ok());
    assert_eq!(csm.len(), 2);

    let res = csm.remove_single_state(43);
    assert!(res.is_ok());
    assert_eq!(csm.len(), 1);
}

#[test]
fn remove_single_state_err_not_found() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    assert_eq!(csm.len(), 1);

    let res = csm.remove_single_state(99);
    assert!(res.is_err());
    assert_eq!(csm.len(), 1);
}

#[test]
fn eval_single_state_error_fail_action() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils_csm::get_test_error_causaloid();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let data = test_utils::get_test_single_data(0.23f64);
    let res = csm.eval_single_state(23, &data);
    assert!(res.is_err())
}

#[test]
fn eval_single_state() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let data = test_utils::get_test_single_data(0.23f64);
    let res = csm.eval_single_state(23, &data);
    assert!(res.is_err())
}

#[test]
fn eval_single_state_error_non_deter() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils_csm::get_test_probabilistic_causaloid();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let data = test_utils::get_test_single_data(0.23f64);
    let res = csm.eval_single_state(id, &data);
    assert!(res.is_err())
}

#[test]
fn eval_single_state_success_fires_action() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(); // Returns Deterministic(true)

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action(); // Succeeds
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    // Data that makes the state active
    let eval_data = test_utils::get_test_single_data(0.60f64);
    // Use the correct ID
    let res = csm.eval_single_state(id, &eval_data);
    assert!(res.is_ok());
}

// Test for the case where the state is not active, so the action is not fired.
#[test]
fn eval_single_state_success_inactive_no_action() {
    let id = 42;
    let version = 1;
    // Use data that makes the state inactive (0.23 < 0.55 threshold)
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(); // Returns Deterministic(false)

    let cs = CausalState::new(id, version, data, causaloid, None);
    // Use an action that would fail to prove it's not being called.
    let ca = test_utils_csm::get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    // Data that makes the state inactive
    let eval_data = test_utils::get_test_single_data(0.23f64);
    // Use the correct ID
    let res = csm.eval_single_state(id, &eval_data);
    // Should be Ok because the state is inactive, so the failing action is never fired.
    assert!(res.is_ok());
}

// Test for the case where the state does not exist.
#[test]
fn eval_single_state_error_not_found() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let eval_data = test_utils::get_test_single_data(0.23f64);
    // Use a non-existent ID
    let res = csm.eval_single_state(99, &eval_data);
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("State 99 does not exist")
    );
}

// This test covers the case where the state evaluation itself fails.
// It adds coverage for the branch you identified.
#[test]
fn eval_single_state_error_eval_fails() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::Numerical(0.23f64);
    // This causaloid's causal_fn always returns an error.
    let causaloid = test_utils_csm::get_test_error_causaloid();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action(); // Action won't be reached
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let eval_data = test_utils::get_test_single_data(0.23f64);
    // Use the correct ID to ensure we get past the 'not found' check.
    let res = csm.eval_single_state(id, &eval_data);

    // The result should be an error because state.eval_with_data failed.
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("CSM Causal Error"));
}

// This test covers the case where the action fails to fire.
#[test]
fn eval_single_state_error_action_fails() {
    let id = 42;
    let version = 1;
    // Use data that makes the state active (0.60 > 0.55 threshold in test_causaloid)
    let data = PropagatingEffect::Numerical(0.60f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(); // Returns Deterministic(true)

    let cs = CausalState::new(id, version, data, causaloid, None);
    // Use an action that is designed to fail.
    let ca = test_utils_csm::get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    // Data that will make the state active
    let eval_data = test_utils::get_test_single_data(0.60f64);
    // Use the correct ID
    let res = csm.eval_single_state(id, &eval_data);

    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("CSM Action Error: ActionError: Error"));
}

#[test]
fn eval_single_state_uncertain_bool_success() {
    let id = 1;
    let version = 1;
    let data = PropagatingEffect::UncertainBool(Uncertain::<bool>::point(true));
    let causaloid = test_utils::get_test_causaloid_uncertain_bool();
    let params = Some(UncertainParameter::new(0.9, 0.99, 0.01, 100));

    let cs = CausalState::new(id, version, data, causaloid, params);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let eval_data = PropagatingEffect::Numerical(0.6);
    let res = csm.eval_single_state(id, &eval_data);
    assert!(res.is_ok());
}

#[test]
fn eval_single_state_uncertain_float_success() {
    let id = 1;
    let version = 1;
    let data = PropagatingEffect::UncertainFloat(Uncertain::<f64>::point(0.99));
    let causaloid = test_utils::get_test_causaloid_uncertain_float();
    let params = Some(UncertainParameter::new(0.9, 0.99, 0.01, 100));

    let cs = CausalState::new(id, version, data, causaloid, params);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action, None);

    let eval_data = PropagatingEffect::Numerical(0.6);
    let res = csm.eval_single_state(id, &eval_data);
    assert!(res.is_ok());
}
