/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{ActionError, CausalityError, CsmError};
use deep_causality_uncertain::UncertainError;
use std::error::Error;

#[test]
fn test_csm_error_forbidden() {
    let explanation = "Action is forbidden because it violates a core principle.".to_string();
    let csm_err = CsmError::Forbidden(explanation.clone());

    // Test Display
    let expected =
        "CSM Forbidden: Action is forbidden because it violates a core principle.".to_string();
    assert_eq!(csm_err.to_string(), expected);

    // Test Debug
    assert!(format!("{:?}", csm_err).contains(&format!("Forbidden(\"{}\")", explanation)));

    // Test source - should be None for Forbidden
    assert!(csm_err.source().is_none());
}

#[test]
fn test_csm_error_from_action_error() {
    let action_err = ActionError("Test action error".to_string());
    let csm_err: CsmError = action_err.into();

    // Test Display
    assert_eq!(
        format!("{}", csm_err),
        "CSM Action Error: ActionError: Test action error"
    );

    // Test Debug
    assert!(format!("{:?}", csm_err).contains("Action(ActionError(\"Test action error\"))"));

    // Test source
    let source = csm_err.source().unwrap();
    assert_eq!(source.to_string(), "ActionError: Test action error");
}

#[test]
fn test_csm_error_from_causality_error() {
    let causality_err = CausalityError::new(deep_causality::CausalityErrorEnum::Custom(
        "Test causality error".to_string(),
    ));
    let csm_err: CsmError = causality_err.into();

    // Test Display
    assert!(csm_err.to_string().contains("Test causality error"));

    // Test Debug
    assert!(format!("{:?}", csm_err).contains("Test causality error"));

    // Test source
    let source = csm_err.source().unwrap();
    assert!(source.to_string().contains("Test causality error"));
}

#[test]
fn test_csm_error_from_uncertain_error() {
    let uncertain_error = UncertainError::UnsupportedTypeError("error".to_string());
    let csm_err: CsmError = uncertain_error.into();

    // Test Display
    assert_eq!(
        format!("{}", csm_err),
        "CSM Uncertain Error: Unsupported type: error"
    );
}
