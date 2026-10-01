/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils;
use deep_causality::utils_test::test_utils_csm;
use deep_causality::{CSM, CausalState, PropagatingEffect, UncertainParameter};

#[test]
fn eval_all_states() {
    let id = 42;
    let version = 1;
    let data = test_utils::get_test_single_data(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    dbg!(&res);
    assert!(res.is_ok())
}

#[test]
fn update_all_states() {
    let id = 42;
    let version = 1;
    let data = test_utils::get_test_single_data(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid.clone(), None);
    let ca = test_utils_csm::get_test_action();

    let state_actions = &[(&cs, &ca)];

    let csm = CSM::new(state_actions);

    assert_eq!(csm.len(), 1);
    let data = test_utils::get_test_single_data(0.23f64);
    let cs2 = CausalState::new(2, 2, data, causaloid, None);
    let ca2 = test_utils_csm::get_test_action();

    let state_actions = &[(&cs, &ca), (&cs2, &ca2)];

    let res = csm.update_all_states(state_actions);
    dbg!(&res);
    assert!(res.is_ok());

    assert_eq!(csm.len(), 2)
}

#[test]
fn eval_all_states_success_inactive_state() {
    let id = 42;
    let version = 1;
    let data = test_utils::get_test_single_data(0.23f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);
    let res = csm.eval_all_states();
    dbg!(&res);
    assert!(res.is_ok())
}

#[test]
fn eval_all_states_success_active_state_fires_action() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.60f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    dbg!(&res);
    assert!(res.is_ok());
}

#[test]
fn eval_all_states_error_eval_fails() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(true);
    let causaloid = test_utils_csm::get_test_error_causaloid();

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    dbg!(&res);

    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    // The error message depends on validation.
    // Since we pass bool, it matches expected. The error is from causaloid itself "Error".
    assert!(err_msg.contains("CSM Causal Error"));
}

#[test]
fn eval_all_states_error_action_fails() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.60f64);
    let causaloid = test_utils::get_test_causaloid_deterministic(23);

    let cs = CausalState::new(id, version, data, causaloid, None);
    let ca = test_utils_csm::get_test_error_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    dbg!(&res);

    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("CSM Action Error: ActionError: Error"));
}

#[test]
fn eval_all_states_error_non_deterministic() {
    let id = 42;
    let version = 1;
    let data = PropagatingEffect::from_value(0.23f64);
    // get_test_probabilistic_causaloid returns f64 which is not CsmEvaluable.
    // We use get_test_causaloid_uncertain_float instead.
    let causaloid = test_utils::get_test_causaloid_uncertain_float();
    let params = Some(UncertainParameter::new(0.9, 0.99, 0.01, 100));

    let cs = CausalState::new(id, version, data, causaloid, params);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    dbg!(&res);
    assert!(res.is_ok());
}

#[test]
fn eval_all_states_uncertain_bool_success() {
    let id = 1;
    let version = 1;
    let data = PropagatingEffect::from_value(0.6);
    let causaloid = test_utils::get_test_causaloid_uncertain_bool();
    let params = Some(UncertainParameter::new(0.9, 0.99, 0.01, 100));

    let cs = CausalState::new(id, version, data, causaloid, params);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    dbg!(&res);
    assert!(res.is_ok());
}

#[test]
fn eval_all_states_uncertain_float_success() {
    let id = 1;
    let version = 1;
    let data = PropagatingEffect::from_value(0.6);
    let causaloid = test_utils::get_test_causaloid_uncertain_float();
    let params = Some(UncertainParameter::new(0.9, 0.99, 0.01, 100));

    let cs = CausalState::new(id, version, data, causaloid, params);
    let ca = test_utils_csm::get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    dbg!(&res);
    assert!(res.is_ok());
}

// Eight states inserted out of order: an unordered map returns them sorted by chance once in
// 40,320 runs, so the assertion pins the ordering rather than the hash seed.
#[test]
fn eval_all_states_returns_every_effect_in_ascending_id_order() {
    let states: Vec<_> = [9usize, 2, 7, 4, 1, 8, 3, 6]
        .into_iter()
        .map(|id| {
            CausalState::new(
                id,
                1,
                test_utils::get_test_single_data(0.60f64),
                test_utils::get_test_causaloid_deterministic(id as u64),
                None,
            )
        })
        .collect();
    let ca = test_utils_csm::get_test_action();
    let pairs: Vec<_> = states.iter().map(|s| (s, &ca)).collect();
    let csm = CSM::new(&pairs);

    let results = csm.eval_all_states().expect("every state evaluates");

    let ids: Vec<usize> = results.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, vec![1, 2, 3, 4, 6, 7, 8, 9]);
    for (id, effect) in &results {
        assert_eq!(effect.value(), Some(&true));
        let expected =
            format!("CSM state {id} (version 1): active; fired 'Test action' (version 1)");
        assert_eq!(effect.logs().messages().last(), Some(expected.as_str()));
    }
}
