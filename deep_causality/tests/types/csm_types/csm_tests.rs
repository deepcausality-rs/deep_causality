/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::*;

use deep_causality::utils_test::test_utils;

// Standard action that succeeds
fn state_action() -> Result<(), ActionError> {
    Ok(())
}

fn get_test_action() -> CausalAction {
    CausalAction::new(state_action, "Test action", 1)
}

fn get_test_error_action() -> CausalAction {
    fn err_state_action() -> Result<(), ActionError> {
        Err(ActionError("Error".to_string()))
    }

    CausalAction::new(err_state_action, "Test action", 1)
}

// Causaloid that returns a non-deterministic effect
fn get_test_probabilistic_causaloid() -> BaseCausaloid {
    fn causal_fn(_: &Evidence) -> Result<PropagatingEffect, CausalityError> {
        Ok(PropagatingEffect::Probabilistic(0.5))
    }
    Causaloid::new(99, causal_fn, "Probabilistic Causaloid")
}

fn get_test_error_causaloid() -> BaseCausaloid {
    fn causal_fn(_: &Evidence) -> Result<PropagatingEffect, CausalityError> {
        Err(CausalityError::new("Error".to_string()))
    }
    Causaloid::new(78, causal_fn, "Probabilistic Causaloid")
}

#[test]
fn test_new() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();

    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1)
}

#[test]
fn test_is_empty() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();

    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert!(!csm.is_empty())
}

#[test]
fn add_single_state() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid.clone());
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);
    let data = Evidence::Numerical(0.23f64);
    let cs2 = CausalState::new(2, 2, data, causaloid);
    let ca2 = get_test_action();
    let state_action = (cs2, ca2);

    let res = csm.add_single_state(43, state_action);

    assert!(res.is_ok());
    assert_eq!(csm.len(), 2);
}

#[test]
fn add_single_state_err_already_exists() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid.clone());
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);
    let data = Evidence::Numerical(0.23f64);
    let cs2 = CausalState::new(2, 2, data, causaloid);
    let ca2 = get_test_action();
    let state_action = (cs2, ca2);

    let res = csm.add_single_state(id, state_action);

    assert!(res.is_err());
    assert_eq!(csm.len(), 1);
}

#[test]
fn update_single_state() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();

    let state_action = &[(&cs, &ca)];

    let csm = CSM::new(state_action);
    assert_eq!(csm.len(), 1);

    let id = 44;
    let version = 1;
    let data = Evidence::Numerical(0.7f64);

    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();

    let state_action = (cs, ca);

    let res = csm.update_single_state(42, state_action);
    assert!(res.is_ok());
    assert_eq!(csm.len(), 1);
}

#[test]
fn update_single_state_err_not_found() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.update_single_state(99, (cs, ca));
    assert!(res.is_err());
}

#[test]
fn remove_single_state() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data.clone(), causaloid.clone());
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);

    let cs2 = CausalState::new(2, 2, data, causaloid);
    let ca2 = get_test_action();
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
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);

    let res = csm.remove_single_state(99);
    assert!(res.is_err());
    assert_eq!(csm.len(), 1);
}

#[test]
fn eval_single_state_error_fail_action() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = get_test_error_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let data = test_utils::get_test_single_data(0.23f64);
    let res = csm.eval_single_state(23, data);
    assert!(res.is_err())
}

#[test]
fn eval_all_states() {
    let id = 42;
    let version = 1;
    let data = test_utils::get_test_single_data(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    assert!(res.is_ok())
}

#[test]
fn update_all_states() {
    let id = 42;
    let version = 1;
    let data = test_utils::get_test_single_data(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid.clone());
    let ca = get_test_action();

    let state_actions = &[(&cs, &ca)];

    let csm = CSM::new(state_actions);

    assert_eq!(csm.len(), 1);
    let data = test_utils::get_test_single_data(0.23f64);
    let cs2 = CausalState::new(2, 2, data, causaloid);
    let ca2 = get_test_action();

    let state_actions = &[(&cs, &ca), (&cs2, &ca2)];

    let res = csm.update_all_states(state_actions);
    assert!(res.is_ok());

    assert_eq!(csm.len(), 2)
}

#[test]
fn eval_single_state() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let data = test_utils::get_test_single_data(0.23f64);
    let res = csm.eval_single_state(23, data);
    assert!(res.is_err())
}

#[test]
fn eval_single_state_error_non_deter() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = get_test_probabilistic_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let data = test_utils::get_test_single_data(0.23f64);
    let res = csm.eval_single_state(id, data);
    assert!(res.is_err())
}

#[test]
fn eval_single_state_success_fires_action() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid(); // Returns Deterministic(true)

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action(); // Succeeds
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    // Data that makes the state active
    let eval_data = test_utils::get_test_single_data(0.23f64);
    // Use the correct ID
    let res = csm.eval_single_state(id, eval_data);
    assert!(res.is_ok());
}

// Test for the case where the state is not active, so the action is not fired.
#[test]
fn eval_single_state_success_inactive_no_action() {
    let id = 42;
    let version = 1;
    // Use data that makes the state inactive (0.23 < 0.55 threshold)
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid(); // Returns Deterministic(false)

    let cs = CausalState::new(id, version, data, causaloid);
    // Use an action that would fail to prove it's not being called.
    let ca = get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    // Data that makes the state inactive
    let eval_data = test_utils::get_test_single_data(0.23f64);
    // Use the correct ID
    let res = csm.eval_single_state(id, eval_data);
    // Should be Ok because the state is inactive, so the failing action is never fired.
    assert!(res.is_ok());
}

// Test for the case where the state does not exist.
#[test]
fn eval_single_state_error_not_found() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let eval_data = test_utils::get_test_single_data(0.23f64);
    // Use a non-existent ID
    let res = csm.eval_single_state(99, eval_data);
    assert!(res.is_err());
    assert!(res.unwrap_err().0.contains("State 99 does not exist"));
}

// This test covers the case where the state evaluation itself fails.
// It adds coverage for the branch you identified.
#[test]
fn eval_single_state_error_eval_fails() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    // This causaloid's causal_fn always returns an error.
    let causaloid = get_test_error_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action(); // Action won't be reached
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let eval_data = test_utils::get_test_single_data(0.23f64);
    // Use the correct ID to ensure we get past the 'not found' check.
    let res = csm.eval_single_state(id, eval_data);

    // The result should be an error because state.eval_with_data failed.
    assert!(res.is_err());
    let err_msg = res.unwrap_err().0;
    assert!(err_msg.contains(&format!("CSM[eval]: Error evaluating state {id}")));
}

// This test covers the case where the action fails to fire.
#[test]
fn eval_single_state_error_action_fails() {
    let id = 42;
    let version = 1;
    // Use data that makes the state active (0.60 > 0.55 threshold in test_causaloid)
    let data = Evidence::Numerical(0.60f64);
    let causaloid = test_utils::get_test_causaloid(); // Returns Deterministic(true)

    let cs = CausalState::new(id, version, data, causaloid);
    // Use an action that is designed to fail.
    let ca = get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    // Data that will make the state active
    let eval_data = test_utils::get_test_single_data(0.60f64);
    // Use the correct ID
    let res = csm.eval_single_state(id, eval_data);

    assert!(res.is_err());
    let err_msg = res.unwrap_err().0;
    assert!(err_msg.contains(&format!("CSM[eval]: Failed to fire action for state {id}")));
}

// I've renamed the original test to be more descriptive.
// It correctly tests the success path where a state is inactive.
#[test]
fn eval_all_states_success_inactive_state() {
    let id = 42;
    let version = 1;
    // Data that makes the state inactive (0.23 < 0.55 threshold)
    let data = test_utils::get_test_single_data(0.23f64);
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    assert!(res.is_ok())
}

// New test for the success path where a state is active and the action fires.
#[test]
fn eval_all_states_success_active_state_fires_action() {
    let id = 42;
    let version = 1;
    // Data that makes the state active (0.6 > 0.55 threshold)
    let data = Evidence::Numerical(0.60f64);
    let causaloid = test_utils::get_test_causaloid(); // Returns Deterministic(true)

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action(); // Succeeds
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    assert!(res.is_ok());
}

// New test for the first error branch: state evaluation fails.
#[test]
fn eval_all_states_error_eval_fails() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    // This causaloid's causal_fn always returns an error.
    let causaloid = get_test_error_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action(); // Action won't be reached
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();

    assert!(res.is_err());
    let err_msg = res.unwrap_err().0;
    assert!(err_msg.contains(&format!("CSM[eval]: Error evaluating state {id}")));
}

// New test for the second error branch: action firing fails.
#[test]
fn eval_all_states_error_action_fails() {
    let id = 42;
    let version = 1;
    // Use data that makes the state active (0.60 > 0.55 threshold)
    let data = Evidence::Numerical(0.60f64);
    let causaloid = test_utils::get_test_causaloid(); // Returns Deterministic(true)

    let cs = CausalState::new(id, version, data, causaloid);
    // Use an action that is designed to fail.
    let ca = get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();

    assert!(res.is_err());
    let err_msg = res.unwrap_err().0;
    assert!(err_msg.contains(&format!("CSM[eval]: Failed to fire action for state {id}")));
}

// New test for the third error branch: non-deterministic effect.
#[test]
fn eval_all_states_error_non_deterministic() {
    let id = 42;
    let version = 1;
    let data = Evidence::Numerical(0.23f64);
    let causaloid = get_test_probabilistic_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    assert!(res.is_err());
    let err_msg = res.unwrap_err().0;
    assert!(err_msg.contains("Invalid non-deterministic effect"));
    assert!(err_msg.contains(&format!("for state {id}")));
}
