/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils_csm;
use deep_causality::*;

#[test]
#[should_panic(expected = "EffectEthos must be verified before being used in a CSM.")]
fn test_new_csm_with_unverified_ethos_panics() {
    let state = CausalState::new(
        1,
        1,
        PropagatingEffect::none(),
        test_utils_csm::get_test_causaloid(true),
        None,
    );
    let action = CausalAction::new(|| Ok(()), "", 1);
    let ethos = test_utils_csm::get_effect_ethos(false, true); // Not verified

    CSM::new(&[(&state, &action)], Some((ethos, &["test_tag"])));
}

#[test]
fn test_eval_single_state_with_ethos_permitted_verdict() {
    let state = CausalState::new(
        1,
        1,
        PropagatingEffect::none(),
        test_utils_csm::get_test_causaloid(true),
        None,
    );
    let action = test_utils_csm::get_test_action_with_tracker();
    let ethos = test_utils_csm::get_effect_ethos(true, false); // Verified, Impermissible

    let csm = CSM::new(&[(&state, &action)], Some((ethos, &["test_tag"])));
    let res = csm.eval_single_state(1, &PropagatingEffect::from_boolean(true));
    dbg!(&res);

    assert!(res.is_ok());
}

#[test]
fn test_eval_single_state_with_ethos_permissible_verdict() {
    let state = CausalState::new(
        1,
        1,
        PropagatingEffect::none(),
        test_utils_csm::get_test_causaloid(true),
        None,
    );
    let impermissible = false;
    let action = test_utils_csm::get_test_action_with_tracker();
    let ethos = test_utils_csm::get_effect_ethos(true, impermissible); // Verified, Permissible

    let csm = CSM::new(&[(&state, &action)], Some((ethos, &["test_tag"])));
    // we have to use Boolean(false) b/c the causaloid inverts it to true, then the CSM evaluation starts.
    let res = csm.eval_single_state(1, &PropagatingEffect::from_boolean(false));
    dbg!(&res);
    assert!(res.is_ok());
}

#[test]
fn test_eval_single_state_with_ethos_impermissible_verdict() {
    let state = CausalState::new(
        1,
        1,
        PropagatingEffect::none(),
        test_utils_csm::get_test_causaloid(true),
        None,
    );

    let impermissible = true;
    let action = test_utils_csm::get_test_action_with_tracker();
    let ethos = test_utils_csm::get_effect_ethos(true, impermissible); // Verified, Permissible

    let csm = CSM::new(&[(&state, &action)], Some((ethos, &["test_tag"])));
    // we have to use Boolean(false) b/c the causaloid inverts it to true, then the CSM evaluation starts.
    let res = csm.eval_single_state(1, &PropagatingEffect::from_boolean(false));
    dbg!(&res);
    assert!(res.is_err());
    let err = res.unwrap_err().to_string();
    assert!(err.contains("Impermissible"));
}

#[test]
fn test_eval_single_state_with_ethos_missing_context_errs() {
    let state = CausalState::new(
        1,
        1,
        PropagatingEffect::none(),
        test_utils_csm::get_test_causaloid(false),
        None,
    ); // No context
    let action = CausalAction::new(|| Ok(()), "", 1);
    let ethos = test_utils_csm::get_effect_ethos(true, true);

    let csm = CSM::new(&[(&state, &action)], Some((ethos, &["test_tag"])));
    let res = csm.eval_single_state(1, &PropagatingEffect::none());
    dbg!(&res);
    assert!(res.is_err());
    let err = res.unwrap_err().to_string();
    assert!(err.contains("Expected Deterministic(bool), found None"));
}

#[test]
fn test_eval_all_states() {
    let state = CausalState::new(
        1,
        1,
        PropagatingEffect::from_boolean(true),
        test_utils_csm::get_test_causaloid(true),
        None,
    );
    let action = test_utils_csm::get_test_action_with_tracker();
    let ethos = test_utils_csm::get_effect_ethos(true, false); // Verified, Permissible

    let csm = CSM::new(&[(&state, &action)], Some((ethos, &["test_tag"])));
    let res = csm.eval_all_states();
    dbg!(&res);
    assert!(res.is_ok());
}

#[test]
fn test_eval_all_state_with_ethos_permissible_verdict() {
    let state = CausalState::new(
        1,
        1,
        // we have to use Boolean(false) b/c the causaloid inverts it to true, then the CSM evaluation starts.
        PropagatingEffect::from_boolean(false),
        test_utils_csm::get_test_causaloid(true),
        None,
    );

    let impermissible = false;
    let action = test_utils_csm::get_test_action_with_tracker();
    let ethos = test_utils_csm::get_effect_ethos(true, impermissible); // Verified, Permissible

    let csm = CSM::new(&[(&state, &action)], Some((ethos, &["test_tag"])));
    let res = csm.eval_all_states();
    dbg!(&res);
    assert!(res.is_ok());
}

#[test]
fn test_eval_all_state_with_ethos_impermissible_verdict() {
    let state = CausalState::new(
        1,
        1,
        // we have to use Boolean(false) b/c the causaloid inverts it to true, then the CSM evaluation starts.
        PropagatingEffect::from_boolean(false),
        test_utils_csm::get_test_causaloid(true),
        None,
    );

    let impermissible = true;
    let action = test_utils_csm::get_test_action_with_tracker();
    let ethos = test_utils_csm::get_effect_ethos(true, impermissible); // Verified, Permissible

    let csm = CSM::new(&[(&state, &action)], Some((ethos, &["test_tag"])));
    let res = csm.eval_all_states();
    dbg!(&res);
    assert!(res.is_err());
    let e = res.unwrap_err();
    assert!(e.to_string().contains("Impermissible"));
}

#[test]
fn test_eval_all_states_non_deterministic_error() {
    // State 1: OK
    let state_1 = CausalState::new(
        1,
        1,
        PropagatingEffect::none(),
        test_utils_csm::get_test_causaloid(true),
        None,
    ); // No context
    let action_1 = test_utils_csm::get_test_action();

    // State 2: Non-deterministic
    let action_2 = test_utils_csm::get_test_action();
    let causaloid_2 = test_utils_csm::get_test_causaloid(true);
    let state_2 = CausalState::new(2, 2, PropagatingEffect::none(), causaloid_2, None);

    let ethos = test_utils_csm::get_effect_ethos(true, true);

    let csm = CSM::new(
        &[(&state_1, &action_1), (&state_2, &action_2)],
        Some((ethos, &["test_tag"])),
    );

    let res = csm.eval_all_states();
    dbg!(&res);

    assert!(res.is_err());

    let e = res.unwrap_err();
    assert!(
        e.to_string()
            .contains("Expected Deterministic(bool), found None")
    );
}

#[test]
fn test_eval_all_states_missing_context_error() {
    let state = CausalState::new(
        1,
        1,
        PropagatingEffect::from_boolean(true),
        test_utils_csm::get_test_causaloid(false),
        None,
    ); // No context
    let action = CausalAction::new(|| Ok(()), "", 1);
    let ethos = test_utils_csm::get_effect_ethos(true, true);

    let csm = CSM::new(&[(&state, &action)], Some((ethos, &["test_tag"])));
    let res = csm.eval_all_states();
    dbg!(&res);

    assert!(res.is_err());
    if let Err(CsmError::Action(e)) = res {
        assert_eq!(
            e.to_string(),
            "ActionError: Cannot evaluate action with ethos because state context is missing."
        );
    } else {
        panic!("Expected CsmError::Action");
    }
}
