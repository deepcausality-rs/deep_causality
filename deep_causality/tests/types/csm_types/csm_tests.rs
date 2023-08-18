// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::prelude::{ActionError, CausalAction, CausalState, CSM};
use deep_causality::utils::test_utils;

fn state_action() -> Result<(), ActionError> {
    println!("Detected something and acted upon");

    Ok(())
}

fn get_test_action() -> CausalAction {
    let func = state_action;
    let descr = "Test action that prints something";
    let version = 1;

    CausalAction::new(func, descr, version)
}

#[test]
fn test_new()
{
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = &test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();

    let state_action = &[(&cs, &ca)];

    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1)
}

#[test]
fn add_single_state()
{
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, &causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);

    let cs2 = CausalState::new(2, 2, data, &causaloid);
    let ca2 = get_test_action();
    let state_action = (&cs2, &ca2);

    let res = csm.add_single_state(43, state_action);

    assert!(res.is_ok());
    assert_eq!(csm.len(), 2);
}


#[test]
fn add_single_state_err_already_exists()
{
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, &causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);

    let cs2 = CausalState::new(2, 2, data, &causaloid);
    let ca2 = get_test_action();
    let state_action = (&cs2, &ca2);

    let res = csm.add_single_state(id, state_action);

    assert!(res.is_err());
    assert_eq!(csm.len(), 1);
}

#[test]
fn remove_single_state()
{
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, &causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);

    let cs2 = CausalState::new(2, 2, data, &causaloid);
    let ca2 = get_test_action();
    let state_action = (&cs2, &ca2);

    let res = csm.add_single_state(43, state_action);

    assert!(res.is_ok());
    assert_eq!(csm.len(), 2);

    let res = csm.remove_single_state(43);
    assert!(res.is_ok());
    assert_eq!(csm.len(), 1);
}

#[test]
fn remove_single_state_err_not_found()
{
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, &causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1);

    let res = csm.remove_single_state(99);
    assert!(res.is_err());
    assert_eq!(csm.len(), 1);
}

#[test]
fn eval_single_state()
{
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = &test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let data = 0.89f64;
    let res = csm.eval_single_state(id, data);
    assert!(res.is_ok())
}

#[test]
fn eval_single_state_err_not_found()
{
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = &test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_single_state(23, data);
    assert!(res.is_err())
}

#[test]
fn update_single_state()
{
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = &test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();

    let state_action = &[(&cs, &ca)];

    let csm = CSM::new(state_action);

    let data = 0.89f64;
    let res = csm.eval_single_state(id, data);
    assert!(res.is_ok())
}

#[test]
fn update_single_state_err_not_found()
{
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = &test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_single_state(99, data);
    assert!(res.is_err())
}

#[test]
fn eval_all_states()
{
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = &test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();
    let state_action = &[(&cs, &ca)];
    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();
    assert!(res.is_ok())
}

#[test]
fn update_all_states()
{
    let id = 42;
    let version = 1;
    let data = 0.23f64;
    let causaloid = &test_utils::get_test_causaloid();

    let cs = CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();

    let state_actions = &[(&cs, &ca)];

    let csm = CSM::new(state_actions);

    assert_eq!(csm.len(), 1);

    let cs2 = CausalState::new(2, 2, data, causaloid);
    let ca2 = get_test_action();

    let state_actions = &[(&cs, &ca), (&cs2, &ca2)];

    csm.update_all_states(state_actions);

    assert_eq!(csm.len(), 2)
}