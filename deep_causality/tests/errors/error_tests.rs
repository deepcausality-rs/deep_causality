// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::errors::{ActionError, AdjustmentError, ContextIndexError, PropagateError, UpdateError};
use deep_causality::prelude::{BuildError, CausalityGraphError, CausalityError};

#[test]
fn test_build_error() {
    let x = 1;
    let result: Result<usize, BuildError> = Err(BuildError(
        format!("unexpected number {}", x),
    ));
    let build_error = result.unwrap_err();
    assert_eq!(build_error.to_string(), format!("BuildError: unexpected number {}", 1));
}

#[test]
fn test_causality_graph_error() {
    let result: Result<usize, CausalityGraphError> = Err(CausalityGraphError(
        format!("unexpected cause"),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("CausalityGraphError: unexpected cause"));
}

#[test]
fn test_context_index_error() {
    let result: Result<usize, ContextIndexError> = Err(ContextIndexError(
        format!("unexpected cause"),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("ContextIndexError: unexpected cause"));

}

#[test]
fn test_causality_error() {
    let result: Result<usize, CausalityError> = Err(CausalityError(
        format!("unexpected cause"),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("CausalityError: unexpected cause"));
}

#[test]
fn test_adjustment_error() {
    let result: Result<usize, AdjustmentError> = Err(AdjustmentError(
        format!("unexpected issue"),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("AdjustmentError: unexpected issue"));
}

#[test]
fn test_propagate_error() {
    let result: Result<usize, PropagateError> = Err(PropagateError(
        format!("unexpected issue"),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("PropagateError: unexpected issue"));
}

#[test]
fn test_update_error() {
    let result: Result<usize, UpdateError> = Err(UpdateError(
        format!("unexpected issue"),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("UpdateError: unexpected issue"));
}

#[test]
fn test_action_error() {
    let result: Result<usize, ActionError> = Err(ActionError(
        format!("unexpected issue"),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("ActionError: unexpected issue"));
}
