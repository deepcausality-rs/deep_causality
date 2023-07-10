// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::csm::CSM;
use deep_causality::prelude::{ActionError, CausalAction, CausalState};
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
    let data = &[0.23f64];
    let causaloid = &test_utils::get_test_causaloid();

    let cs =  CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();

    let state_action = &[(&cs, &ca)];

    let csm = CSM::new(state_action);

    assert_eq!(csm.len(), 1)
}

#[test]
fn test_eval()
{
    let id = 42;
    let version = 1;
    let data = &[0.23f64];
    let causaloid = &test_utils::get_test_causaloid();

    let cs =  CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();

    let state_action = &[(&cs, &ca)];

    let csm = CSM::new(state_action);

    let res = csm.eval_all_states();

    assert!(res.is_ok())

}

#[test]
fn test_update()
{
    let id = 42;
    let version = 1;
    let data = &[0.23f64];
    let causaloid = &test_utils::get_test_causaloid();

    let cs =  CausalState::new(id, version, data, causaloid);
    let ca = get_test_action();

    let state_actions = &[(&cs, &ca)];

    let csm = CSM::new(state_actions);

    assert_eq!(csm.len(), 1);

    let cs2 =  CausalState::new(2, 2, data, causaloid);
    let ca2 = get_test_action();

    let state_actions = &[(&cs, &ca), (&cs2, &ca2)];

    csm.update_all_states(state_actions);

    assert_eq!(csm.len(), 2)
}