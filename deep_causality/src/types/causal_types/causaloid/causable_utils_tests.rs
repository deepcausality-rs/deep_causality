/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::causal_types::causaloid::causable_utils::{
    convert_input, convert_output, execute_causal_logic,
};
use crate::*;
use std::sync::{Arc, RwLock};

// Mock causal function for testing
fn test_causal_fn(value: bool) -> Result<CausalFnOutput<f64>, CausalityError> {
    if value {
        Ok(CausalFnOutput::new(
            42.0,
            "Success from test_causal_fn".into(),
        ))
    } else {
        Err(CausalityError("Failure from test_causal_fn".to_string()))
    }
}

// Mock contextual causal function for testing
fn test_context_causal_fn(
    value: bool,
    _context: &Arc<RwLock<BaseContext>>,
) -> Result<CausalFnOutput<f64>, CausalityError> {
    if value {
        Ok(CausalFnOutput::new(
            99.0,
            "Success from test_context_causal_fn".into(),
        ))
    } else {
        Err(CausalityError(
            "Failure from test_context_causal_fn".to_string(),
        ))
    }
}

#[test]
fn test_convert_input_success() {
    let effect_val = EffectValue::Boolean(true);
    let id = 1;
    let result = convert_input::<bool>(effect_val, id);

    assert!(result.is_ok());
    assert!(result.value);
    assert!(!result.logs.is_empty());
    let log_str = result.logs.to_string();
    assert!(log_str.contains("Causaloid 1: Incoming effect: Boolean(true)"));
}

#[test]
fn test_convert_input_failure() {
    let effect_val = EffectValue::Numerical(42.0); // Mismatched type
    let id = 2;
    let result = convert_input::<bool>(effect_val, id);

    assert!(result.is_err());
    assert_eq!(result.value, bool::default()); // Should be default on error
    let error = result.error.unwrap();
    // Corrected assertion to match the actual error message from EffectValue::try_from_effect_value
    assert_eq!(
        error.0,
        "Expected Deterministic(bool), found Numerical(42.0)"
    );

    assert!(!result.logs.is_empty());
    let log_str = result.logs.to_string();
    assert!(log_str.contains("Causaloid 2: Incoming effect: Numerical(42.0)"));
    assert!(log_str.contains("Causaloid 2: Input conversion failed"));
}

#[test]
fn test_execute_causal_logic_with_causal_fn_success() {
    let causaloid = BaseCausaloid::<bool, f64>::new(1, test_causal_fn, "test");
    let input = true;
    let result = execute_causal_logic(input, &causaloid);

    assert!(result.is_ok());
    assert_eq!(result.value, 42.0);
    assert!(!result.logs.is_empty());
    assert!(
        result
            .logs
            .to_string()
            .contains("Success from test_causal_fn")
    );
}

#[test]
fn test_execute_causal_logic_with_causal_fn_failure() {
    let causaloid = BaseCausaloid::<bool, f64>::new(1, test_causal_fn, "test");
    let input = false; // This will trigger the error path in the mock function
    let result = execute_causal_logic(input, &causaloid);

    assert!(result.is_err());
    assert_eq!(result.value, f64::default());
    let error = result.error.unwrap();
    assert_eq!(error.0, "Failure from test_causal_fn");
    // Check that the log contains the specific error message part, without "CausalityError:"
    assert!(result.logs.to_string().contains(
        "Causaloid 1: Causal function failed: CausalityError: Failure from test_causal_fn"
    ));
}

#[test]
fn test_execute_causal_logic_with_context_fn_success() {
    let context = Arc::new(RwLock::new(BaseContext::with_capacity(
        0,
        "root context",
        5,
    ))); // Corrected constructor
    let causaloid = BaseCausaloid::<bool, f64>::new_with_context(
        2,
        test_context_causal_fn,
        context,
        "test_context",
    );
    let input = true;
    let result = execute_causal_logic(input, &causaloid);

    assert!(result.is_ok());
    assert_eq!(result.value, 99.0);
    assert!(!result.logs.is_empty());
    assert!(
        result
            .logs
            .to_string()
            .contains("Success from test_context_causal_fn")
    );
}

#[test]
fn test_execute_causal_logic_with_context_fn_failure() {
    let context = Arc::new(RwLock::new(BaseContext::with_capacity(
        0,
        "root context",
        5,
    ))); // Corrected constructor
    let causaloid = BaseCausaloid::<bool, f64>::new_with_context(
        2,
        test_context_causal_fn,
        context,
        "test_context",
    );
    let input = false; // Triggers error
    let result = execute_causal_logic(input, &causaloid);
    dbg!(&result);

    assert!(result.is_err());
    assert_eq!(result.value, f64::default());
    let error = result.error.unwrap();
    assert_eq!(error.0, "Failure from test_context_causal_fn");
    // Check that the log contains the specific error message part, without "CausalityError:"
    assert!(result.logs.to_string().contains(
        "Causaloid 2: Causal function failed: CausalityError: Failure from test_context_causal_fn"
    ));
}

#[test]
fn test_execute_causal_logic_with_context_fn_but_no_context() {
    // Manually construct the causaloid to ensure causal_fn is None
    let causaloid: BaseCausaloid<bool, f64> = Causaloid {
        id: 3,
        causal_type: CausaloidType::Singleton,
        causal_fn: None,
        context_causal_fn: Some(test_context_causal_fn),
        context: None,
        coll_aggregate_logic: None,
        coll_threshold_value: None,
        causal_coll: None,
        causal_graph: None,
        description: "test_no_context".to_string(),
        ty: std::marker::PhantomData,
        _phantom: std::marker::PhantomData,
    };

    let input = true;
    let result = execute_causal_logic(input, &causaloid);

    assert!(result.is_err());
    let error = result.error.unwrap();
    assert_eq!(error.0, "Causaloid::evaluate: context is None");
}

#[test]
fn test_execute_causal_logic_no_function_defined() {
    // Create a causaloid with no causal function.
    let causaloid: BaseCausaloid<bool, f64> = Causaloid {
        id: 4,
        causal_type: CausaloidType::Singleton,
        causal_fn: None,
        context_causal_fn: None,
        context: None,
        coll_aggregate_logic: None,
        coll_threshold_value: None,
        causal_coll: None,
        causal_graph: None,
        description: "no function".to_string(),
        ty: std::marker::PhantomData,
        _phantom: std::marker::PhantomData,
    };

    let input = true;
    let result = execute_causal_logic(input, &causaloid);

    assert!(result.is_err());
    let error = result.error.unwrap();
    assert_eq!(
        error.0,
        "Causaloid 4 is missing both causal_fn and context_causal_fn"
    );
}

#[test]
fn test_convert_output() {
    let output_val = 42.0f64;
    let id = 5;
    let result = convert_output(output_val, id);

    assert!(result.is_ok());
    assert_eq!(result.value, EffectValue::Numerical(42.0));
    assert!(!result.logs.is_empty());
    let log_str = result.logs.to_string();
    assert!(log_str.contains("Causaloid 5: Outgoing effect: Numerical(42.0)"));
}

#[test]
fn test_convert_output_bool() {
    let output_val = true;
    let id = 6;
    let result = convert_output(output_val, id);

    assert!(result.is_ok());
    assert_eq!(result.value, EffectValue::Boolean(true));
    assert!(!result.logs.is_empty());
    let log_str = result.logs.to_string();
    assert!(log_str.contains("Causaloid 6: Outgoing effect: Boolean(true)"));
}
