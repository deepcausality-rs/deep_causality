/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::utils_test::test_utils_csm;
use deep_causality::{
    BaseCausaloid, CSM, CausalAction, CausalEffect, CausalState, Causaloid, PropagatingEffect,
    UncertainParameter,
};
use deep_causality_context::BaseContext;
use std::sync::{Arc, RwLock};

#[test]
fn add_single_state() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid.clone(), None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);
    let data = PropagatingEffect::from_value(0.23f64);
    let cs2 = CausalState::new(2, 2, data, causaloid, None);
    let ca2 = test_utils_csm::get_test_action();
    let state_action = (cs2, ca2);

    let res = csm.add_single_state(state_action);
    dbg!(&res);

    assert!(res.is_ok());
    assert_eq!(csm.len(), 2);
}

#[test]
fn add_single_state_err_already_exists() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid.clone(), None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);
    let data = PropagatingEffect::from_value(0.23f64);
    let cs2 = CausalState::new(id, 2, data, causaloid, None);
    let ca2 = test_utils_csm::get_test_action();
    let state_action = (cs2, ca2);

    let res = csm.add_single_state(state_action);
    dbg!(&res);

    assert!(res.is_err());
    assert_eq!(csm.len(), 1);
}

#[test]
fn update_single_state() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();

    let state_action = &[(&cs, &ca)];

    let csm = CSM::new(state_action);
    assert_eq!(csm.len(), 1);

    let id = 42; // Match existing state ID
    let version = 1;
    let data = PropagatingEffect::from_value(0.7f64);

    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();

    let state_action = (cs, ca);

    let res = csm.update_single_state(state_action);
    dbg!(&res);

    assert!(res.is_ok());
    assert_eq!(csm.len(), 1);
}

#[test]
fn update_single_state_err_not_found() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data.clone(), causaloid.clone(), None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let cs_new = CausalState::new(99, version, data, causaloid, None);
    let res = csm.update_single_state((cs_new, ca));
    dbg!(&res);

    assert!(res.is_err());
}

#[test]
fn remove_single_state() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data.clone(), causaloid.clone(), None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);

    let cs2 = CausalState::new(2, 2, data, causaloid, None);
    let ca2 = test_utils_csm::get_test_action();
    let state_action = (cs2, ca2);

    let res = csm.add_single_state(state_action);

    assert!(res.is_ok());
    assert_eq!(csm.len(), 2);

    let res = csm.remove_single_state(2);
    dbg!(&res);

    assert!(res.is_ok());
    assert_eq!(csm.len(), 1);
}

#[test]
fn remove_single_state_err_not_found() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);

    let res = csm.remove_single_state(99);
    dbg!(&res);

    assert!(res.is_err());
    assert_eq!(csm.len(), 1);
}

// use deep_causality::utils_test::test_utils;
// use deep_causality::utils_test::test_utils_csm;
// use deep_causality::{CSM, CausalState, PropagatingEffect, UncertainParameter};
// use deep_causality_uncertain::Uncertain; // REMOVE

#[test]
fn eval_single_state_error_fail_action() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(true);
    let causaloid = test_utils_csm::get_test_error_causaloid();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let data = PropagatingEffect::from_value(true);
    let res = csm.eval_single_state(23, &data);
    dbg!(&res);

    assert!(res.is_err())
}

#[test]
fn eval_single_state() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let data = test_utils::get_test_single_data(0.23f64);
    let res = csm.eval_single_state(23, &data);
    dbg!(&res);

    assert!(res.is_err())
}

#[test]
fn eval_single_state_success_fires_action() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23); // Returns Boolean(true)

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action(); // Succeeds
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    // Data that makes the state active
    let eval_data = test_utils::get_test_single_data(0.60f64);
    // Use the correct ID
    let res = csm.eval_single_state(id, &eval_data);
    dbg!(&res);

    assert!(res.is_ok());
}

// Test for the case where the state is not active, so the action is not fired.
#[test]
fn eval_single_state_success_inactive_no_action() {
    let id = 42;
    let version = 1;
    // Use data that makes the state inactive (0.23 < 0.55 threshold)
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23); // Returns Boolean(false)

    let cs = CausalState::new(id, version, data, causaloid, None);
    // Use an action that would fail to prove it's not being called.
    let ca = test_utils_csm::get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    // Data that makes the state inactive
    let eval_data = PropagatingEffect::from_value(0.23f64);
    // Use the correct ID
    let res = csm.eval_single_state(id, &eval_data);
    dbg!(&res);
    // Should be Ok because the state is inactive, so the failing action is never fired.
    assert!(res.is_ok());
}

// Test for the case where the state does not exist.
#[test]
fn eval_single_state_error_not_found() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let eval_data = PropagatingEffect::from_value(0.23f64);
    // Use a non-existent ID
    let res = csm.eval_single_state(99, &eval_data);
    dbg!(&res);

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
    let data = PropagatingEffect::from_value(true);
    // This causaloid's causal_fn always returns an error.
    let causaloid = test_utils_csm::get_test_error_causaloid();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action(); // Action won't be reached
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let eval_data = PropagatingEffect::from_value(true);
    // Use the correct ID to ensure we get past the 'not found' check.
    let res = csm.eval_single_state(id, &eval_data);
    dbg!(&res);

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
    let data = PropagatingEffect::from_value(0.60f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23); // Returns Boolean(true)

    let cs = CausalState::new(id, version, data, causaloid, None);
    // Use an action that is designed to fail.
    let ca = test_utils_csm::get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    // Data that will make the state active
    let eval_data = test_utils::get_test_single_data(0.60f64);
    // Use the correct ID
    let res = csm.eval_single_state(id, &eval_data);
    dbg!(&res);

    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("CSM Action Error: ActionError: Error"));
}

// A causaloid whose effect is a `RelayTo` (neither a plain `Value` nor an
// error). This drives the `_ => false` arm in `evaluate_and_fire_action`:
// the state is treated as inactive, so the (failing) action is never fired and
// the overall result is `Ok`.
fn relay_causaloid() -> BaseCausaloid<bool, bool> {
    fn causal_fn(_: bool) -> PropagatingEffect<bool> {
        PropagatingEffect::relay_to(7, CausalEffect::value(true))
    }
    Causaloid::new(55, causal_fn, "Relay Causaloid")
}

#[test]
fn eval_single_state_relay_effect_is_inactive_no_action() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(true);
    let causaloid = relay_causaloid();

    let cs = CausalState::new(id, version, data, causaloid, None);
    // An action that would fail if fired, proving the relay effect is inactive.
    let ca = test_utils_csm::get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let eval_data = PropagatingEffect::from_value(true);
    let res = csm.eval_single_state(id, &eval_data);
    dbg!(&res);

    // The relay value is not active, so the failing action is never fired.
    assert!(res.is_ok());
}

#[test]
fn eval_single_state_uncertain_bool_success() {
    let id = 1;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    let causaloid = test_utils::get_test_causaloid_uncertain_bool();
    let params = Some(UncertainParameter::new(0.9, 0.99, 0.01, 100));

    let cs = CausalState::new(id, version, data, causaloid, params);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let eval_data = PropagatingEffect::from_value(0.6);
    let res = csm.eval_single_state(id, &eval_data);
    dbg!(&res);

    assert!(res.is_ok());
}

#[test]
fn eval_single_state_uncertain_float_success() {
    let id = 1;
    let version = 1;
    let data = PropagatingEffect::from_value(0.99f64);
    let causaloid = test_utils::get_test_causaloid_uncertain_float();
    let params = Some(UncertainParameter::new(0.9, 0.99, 0.01, 100));

    let cs = CausalState::new(id, version, data, causaloid, params);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let eval_data = PropagatingEffect::from_value(0.6);
    let res = csm.eval_single_state(id, &eval_data);
    dbg!(&res);

    assert!(res.is_ok());
}

// The state version (2) and the action version (7) differ, so a swap between them shows.
fn versioned_machine() -> CSM<f64, bool, Arc<RwLock<BaseContext>>> {
    let causaloid = test_utils::get_test_causaloid_deterministic(23);
    let cs = CausalState::new(
        42,
        2,
        PropagatingEffect::from_value(0.23f64),
        causaloid,
        None,
    );
    let ca = CausalAction::new(test_utils_csm::state_action, "open valve", 7);
    CSM::new(&[(&cs, &ca)])
}

#[test]
fn eval_single_state_returns_the_effect_and_records_the_fired_action() {
    let csm = versioned_machine();

    let effect = csm
        .eval_single_state(42, &PropagatingEffect::from_value(0.60f64))
        .expect("state 42 exists and its action succeeds");

    assert_eq!(effect.value(), Some(&true));
    let messages: Vec<&str> = effect.logs().messages().collect();
    assert!(
        messages
            .iter()
            .any(|m| m.starts_with("Causaloid 23: Incoming effect")),
        "the causaloid's own entries are kept: {messages:?}"
    );
    assert_eq!(
        messages.last(),
        Some(&"CSM state 42 (version 2): active; fired 'open valve' (version 7)")
    );
}

#[test]
fn eval_single_state_records_an_inactive_state() {
    let csm = versioned_machine();

    let effect = csm
        .eval_single_state(42, &PropagatingEffect::from_value(0.23f64))
        .expect("state 42 exists");

    assert_eq!(effect.value(), Some(&false));
    assert_eq!(
        effect.logs().messages().last(),
        Some("CSM state 42 (version 2): inactive")
    );
}
