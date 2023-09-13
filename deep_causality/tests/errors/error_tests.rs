// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::errors::{ActionError, AdjustmentError, ContextIndexError, UpdateError};
use deep_causality::prelude::{
    BuildError, CausalGraphIndexError, CausalityError, CausalityGraphError,
};

#[test]
fn test_build_error() {
    let x = 1;
    let result: Result<usize, BuildError> =
        Err(BuildError::new(format!("unexpected number {}", x)));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "BuildError: unexpected number 1".to_string()
    );
}

#[test]
fn test_causality_graph_error() {
    let result: Result<usize, CausalityGraphError> =
        Err(CausalityGraphError::new("unexpected cause".to_string()));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "CausalityGraphError: unexpected cause".to_string()
    );
}

#[test]
fn test_context_index_error() {
    let result: Result<usize, ContextIndexError> =
        Err(ContextIndexError::new("unexpected index".to_string()));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "ContextIndexError: unexpected index".to_string()
    );
}

#[test]
fn test_causal_graph_index_error() {
    let result: Result<usize, CausalGraphIndexError> =
        Err(CausalGraphIndexError::new("unexpected error".to_string()));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "CausalGraphIndexError: unexpected error".to_string()
    );
}

#[test]
fn test_causality_error() {
    let result: Result<usize, CausalityError> =
        Err(CausalityError::new("unexpected cause".to_string()));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "CausalityError: unexpected cause".to_string()
    );
}

#[test]
fn test_adjustment_error() {
    let result: Result<usize, AdjustmentError> =
        Err(AdjustmentError::new("unexpected error".to_string()));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "AdjustmentError: unexpected error".to_string()
    );
}

#[test]
fn test_update_error() {
    let result: Result<usize, UpdateError> = Err(UpdateError::new("unexpected error".to_string()));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "UpdateError: unexpected error".to_string()
    );
}

#[test]
fn test_action_error() {
    let result: Result<usize, ActionError> = Err(ActionError::new("unexpected error".to_string()));
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "ActionError: unexpected error".to_string()
    );
}
