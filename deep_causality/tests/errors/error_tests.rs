// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::errors::{ActionError, AdjustmentError, ContextIndexError, UpdateError};
use deep_causality::prelude::{BuildError, CausalGraphIndexError, CausalityError, CausalityGraphError};

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
        "unexpected cause".to_string(),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("CausalityGraphError: unexpected cause"));
}

#[test]
fn test_context_index_error() {
    let result: Result<usize, ContextIndexError> = Err(ContextIndexError(
        "unexpected cause".to_string(),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("ContextIndexError: unexpected cause"));

}

#[test]
fn test_causal_graph_index_error() {
    let result: Result<usize, CausalGraphIndexError> = Err(CausalGraphIndexError(
        "unexpected cause".to_string(),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("CausalGraphIndexError: unexpected cause"));
}

#[test]
fn test_causality_error() {
    let result: Result<usize, CausalityError> = Err(CausalityError(
        "unexpected cause".to_string(),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("CausalityError: unexpected cause"));
}

#[test]
fn test_adjustment_error() {
    let result: Result<usize, AdjustmentError> = Err(AdjustmentError(
        "unexpected issue".to_string(),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("AdjustmentError: unexpected issue"));
}

#[test]
fn test_update_error() {
    let result: Result<usize, UpdateError> = Err(UpdateError(
        "unexpected issue".to_string(),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("UpdateError: unexpected issue"));
}

#[test]
fn test_action_error() {
    let result: Result<usize, ActionError> = Err(ActionError(
        "unexpected issue".to_string(),
    ));
    let error = result.unwrap_err();
    assert_eq!(error.to_string(), format!("ActionError: unexpected issue"));
}
