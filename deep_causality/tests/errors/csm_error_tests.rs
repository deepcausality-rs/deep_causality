/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::{ActionError, CausalityError, CsmError, DeonticError};
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
fn test_direct_construction() {
    // Test Action variant
    let action_err = ActionError("Direct action".to_string());
    let csm_action_err = CsmError::Action(action_err);
    assert_eq!(
        csm_action_err.to_string(),
        "CSM Action Error: ActionError: Direct action"
    );
    assert!(csm_action_err.source().is_some());

    // Test Deontic variant
    let deontic_err = DeonticError::GraphIsCyclic;
    let csm_deontic_err = CsmError::Deontic(deontic_err.clone());
    assert_eq!(
        csm_deontic_err.to_string(),
        format!("CSM Deontic Error: {}", deontic_err)
    );
    assert!(csm_deontic_err.source().is_some());

    // Test Causal variant
    let causality_err = CausalityError("Direct causal".to_string());
    let csm_causal_err = CsmError::Causal(causality_err);
    assert_eq!(
        csm_causal_err.to_string(),
        "CSM Causal Error: CausalityError: Direct causal"
    );
    assert!(csm_causal_err.source().is_some());
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
fn test_csm_error_from_deontic_error() {
    let deontic_err = DeonticError::MissingContext;
    let csm_err: CsmError = deontic_err.clone().into();

    // Test Display
    let expected_display = format!("CSM Deontic Error: {}", deontic_err);
    assert_eq!(format!("{}", csm_err), expected_display);

    // Test Debug
    assert!(format!("{:?}", csm_err).contains("Deontic(MissingContext)"));

    // Test source
    let source = csm_err.source().unwrap();
    assert_eq!(source.to_string(), deontic_err.to_string());
}

#[test]
fn test_csm_error_from_causality_error() {
    let causality_err = CausalityError("Test causality error".to_string());
    let csm_err: CsmError = causality_err.into();

    // Test Display
    assert_eq!(
        format!("{}", csm_err),
        "CSM Causal Error: CausalityError: Test causality error"
    );

    // Test Debug
    assert!(format!("{:?}", csm_err).contains("Causal(CausalityError(\"Test causality error\"))"));

    // Test source
    let source = csm_err.source().unwrap();
    assert_eq!(source.to_string(), "CausalityError: Test causality error");
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
