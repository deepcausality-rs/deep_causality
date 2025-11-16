/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::utils_test::test_utils_csm::{
    get_effect_ethos, get_test_action, get_test_action_with_tracker, get_test_causaloid,
    get_test_error_action, get_test_error_causaloid, get_test_probabilistic_causaloid,
};
use deep_causality::{DeonticInferable, Identifiable, ProposedAction, TeloidModal};
use std::collections::HashMap;

#[test]
fn test_get_test_error_action() {
    let action = get_test_error_action();
    assert_eq!(action.version(), 1);
    assert_eq!(action.description(), "Test action");
    assert!(action.fire().is_err());
    assert_eq!(action.fire().unwrap_err().to_string(), "ActionError: Error");
}

#[test]
fn test_get_test_probabilistic_causaloid() {
    let causaloid = get_test_probabilistic_causaloid();
    assert_eq!(causaloid.id(), 99);
    assert_eq!(causaloid.description(), "Probabilistic Causaloid");
}

#[test]
fn test_get_test_error_causaloid() {
    let causaloid = get_test_error_causaloid();
    assert_eq!(causaloid.id(), 78);
    assert_eq!(causaloid.description(), "Error Causaloid");
}

#[test]
fn test_get_effect_ethos_verified_impermissible() {
    let ethos = get_effect_ethos(true, true);
    assert!(ethos.is_verified());
    let action = get_test_action();
    let proposed_action = ProposedAction::new(
        action.version() as u64,
        action.description().to_string(),
        HashMap::new(),
    );
    let context = deep_causality::utils_test::test_utils::get_context();
    let verdict = ethos
        .evaluate_action(&proposed_action, &context, &["test_tag"])
        .unwrap();
    assert_eq!(verdict.outcome(), TeloidModal::Impermissible);
}

#[test]
fn test_get_effect_ethos_verified_permissible() {
    let ethos = get_effect_ethos(true, false);
    assert!(ethos.is_verified());
    let action = get_test_action();
    let proposed_action = ProposedAction::new(
        action.version() as u64,
        action.description().to_string(),
        HashMap::new(),
    );
    let context = deep_causality::utils_test::test_utils::get_context();
    let verdict = ethos
        .evaluate_action(&proposed_action, &context, &["test_tag"])
        .unwrap();
    assert_eq!(verdict.outcome(), TeloidModal::Obligatory);
}

#[test]
fn test_get_effect_ethos_unverified() {
    let ethos = get_effect_ethos(false, false);
    assert!(!ethos.is_verified());
    let action = get_test_action();
    let proposed_action = ProposedAction::new(
        action.version() as u64,
        action.description().to_string(),
        HashMap::new(),
    );
    let context = deep_causality::utils_test::test_utils::get_context();
    let err = ethos
        .evaluate_action(&proposed_action, &context, &[])
        .unwrap_err();
    assert_eq!(
        err.to_string(),
        "Deontic inference failed: The TeloidGraph must be frozen before evaluation."
    );
}

#[test]
fn test_get_test_causaloid_with_context() {
    let causaloid = get_test_causaloid(true);
    assert_eq!(causaloid.id(), 1);
    assert_eq!(causaloid.description(), "Inverts any input");
    assert!(causaloid.context().is_some());
}

#[test]
fn test_get_test_causaloid_without_context() {
    let causaloid = get_test_causaloid(false);
    assert_eq!(causaloid.id(), 1);
    assert_eq!(causaloid.description(), "Test Causaloid");
    assert!(causaloid.context().is_none());
}

#[test]
fn test_get_test_action_with_tracker() {
    let action = get_test_action_with_tracker();
    assert_eq!(action.version(), 1);
    assert_eq!(action.description(), "Tracked Action");
    // The action function itself contains the tracker logic, which is not directly accessible here.
    // We can only test that firing it doesn't return an error.
    assert!(action.fire().is_ok());
}
