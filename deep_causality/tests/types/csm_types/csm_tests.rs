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
    let data = Evidence::Numerical(0.88f64);
    let causaloid = test_utils::get_test_causaloid(); // Returns Deterministic(false) for data > 0.5

    let cs = CausalState::new(id, version, data, causaloid);
    // Use an action that would fail to prove it's not being called.
    let ca = get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    // Data that makes the state inactive
    let eval_data = test_utils::get_test_single_data(0.88f64);
    // Use the correct ID
    let res = csm.eval_single_state(id, eval_data);
    assert!(res.is_err());
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
