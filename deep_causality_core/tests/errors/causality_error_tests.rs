/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::{CausalityError, CausalityErrorEnum};
use std::collections::HashSet;
use std::fmt::Write;

#[test]
fn test_new() {
    let error_enum = CausalityErrorEnum::Unspecified;
    let error = CausalityError::new(error_enum.clone());
    assert_eq!(error.0, error_enum);
}

#[test]
fn test_display() {
    let error = CausalityError::new(CausalityErrorEnum::InternalLogicError(
        "join with no parents".to_string(),
    ));
    let mut output = String::new();
    write!(&mut output, "{}", error).unwrap();
    assert_eq!(output, "InternalLogicError(\"join with no parents\")");
}

#[test]
fn test_display_unit_and_index_variants() {
    assert_eq!(
        CausalityError::GraphNotFrozen().to_string(),
        "GraphNotFrozen"
    );
    assert_eq!(
        CausalityError::CausaloidNotFound(7).to_string(),
        "CausaloidNotFound(7)"
    );
}

#[test]
fn test_debug() {
    let error = CausalityError::new(CausalityErrorEnum::TypeConversionError(
        "expected bool, found f64".to_string(),
    ));
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("TypeConversionError"));
    assert!(debug_str.contains("expected bool, found f64"));
    assert!(debug_str.starts_with("CausalityError"));
}

#[test]
fn test_clone() {
    let error1 = CausalityError::new(CausalityErrorEnum::Unspecified);
    let error2 = error1.clone();
    assert_eq!(error1, error2);
}

#[test]
fn test_partial_eq() {
    let error1 = CausalityError::new(CausalityErrorEnum::Unspecified);
    let error2 = CausalityError::InternalLogicError("invariant");
    let error3 = CausalityError::new(CausalityErrorEnum::Unspecified);

    assert_ne!(error1, error2);
    assert_eq!(error1, error3);
}

#[test]
fn test_partial_eq_compares_payloads() {
    assert_eq!(
        CausalityError::CausaloidNotFound(3),
        CausalityError::CausaloidNotFound(3)
    );
    assert_ne!(
        CausalityError::CausaloidNotFound(3),
        CausalityError::CausaloidNotFound(4)
    );
    assert_ne!(CausalityError::Custom("a"), CausalityError::Custom("b"));
    // Same message, different variant: not equal.
    assert_ne!(CausalityError::Custom("m"), CausalityError::ModelError("m"));
}

#[test]
fn test_default() {
    let error: CausalityError = Default::default();
    assert_eq!(error.0, CausalityErrorEnum::default());
    assert_eq!(error, CausalityError::Unspecified());
}

#[test]
fn test_hash() {
    let error1 = CausalityError::new(CausalityErrorEnum::Unspecified);
    let error2 = CausalityError::new(CausalityErrorEnum::Unspecified);

    let mut set = HashSet::new();
    set.insert(error1);
    // Attempting to insert a duplicate
    assert!(!set.insert(error2));
    assert_eq!(set.len(), 1);

    let error3 = CausalityError::InternalLogicError("invariant");
    assert!(set.insert(error3));
    assert_eq!(set.len(), 2);
}

#[cfg(feature = "std")]
#[test]
fn test_std_error_trait() {
    let error = CausalityError::new(CausalityErrorEnum::Unspecified);
    let _err: Box<dyn std::error::Error> = Box::new(error);
}

// --- One constructor per variant ------------------------------------------------------------
//
// Each test pins the variant and the payload its constructor builds, so a constructor that
// returned the wrong variant, or dropped its message, fails here.

#[test]
fn test_ctor_unspecified() {
    assert_eq!(
        CausalityError::Unspecified().0,
        CausalityErrorEnum::Unspecified
    );
}

#[test]
fn test_ctor_internal_logic_error() {
    assert_eq!(
        CausalityError::InternalLogicError("ready node had no fired parents").0,
        CausalityErrorEnum::InternalLogicError("ready node had no fired parents".to_string())
    );
}

#[test]
fn test_ctor_type_conversion_error() {
    assert_eq!(
        CausalityError::TypeConversionError("expected bool, found f64").0,
        CausalityErrorEnum::TypeConversionError("expected bool, found f64".to_string())
    );
}

#[test]
fn test_ctor_value_not_available() {
    assert_eq!(
        CausalityError::ValueNotAvailable().0,
        CausalityErrorEnum::ValueNotAvailable
    );
}

#[test]
fn test_ctor_missing_parameter() {
    assert_eq!(
        CausalityError::MissingParameter("threshold").0,
        CausalityErrorEnum::MissingParameter("threshold".to_string())
    );
}

#[test]
fn test_ctor_unsupported_operation() {
    assert_eq!(
        CausalityError::UnsupportedOperation("aggregate UncertainF64").0,
        CausalityErrorEnum::UnsupportedOperation("aggregate UncertainF64".to_string())
    );
}

#[test]
fn test_ctor_max_steps_exceeded() {
    assert_eq!(
        CausalityError::MaxStepsExceeded().0,
        CausalityErrorEnum::MaxStepsExceeded
    );
}

#[test]
fn test_ctor_unexpected_command() {
    assert_eq!(
        CausalityError::UnexpectedCommand("RelayTo on a singleton input").0,
        CausalityErrorEnum::UnexpectedCommand("RelayTo on a singleton input".to_string())
    );
}

#[test]
fn test_ctor_graph_not_frozen() {
    assert_eq!(
        CausalityError::GraphNotFrozen().0,
        CausalityErrorEnum::GraphNotFrozen
    );
}

#[test]
fn test_ctor_graph_contains_cycle() {
    assert_eq!(
        CausalityError::GraphContainsCycle().0,
        CausalityErrorEnum::GraphContainsCycle
    );
}

#[test]
fn test_ctor_causaloid_not_found() {
    assert_eq!(
        CausalityError::CausaloidNotFound(42).0,
        CausalityErrorEnum::CausaloidNotFound(42)
    );
}

#[test]
fn test_ctor_graph_error() {
    assert_eq!(
        CausalityError::GraphError("edge already exists").0,
        CausalityErrorEnum::GraphError("edge already exists".to_string())
    );
}

#[test]
fn test_ctor_empty_collection() {
    assert_eq!(
        CausalityError::EmptyCollection().0,
        CausalityErrorEnum::EmptyCollection
    );
}

#[test]
fn test_ctor_missing_context() {
    assert_eq!(
        CausalityError::MissingContext().0,
        CausalityErrorEnum::MissingContext
    );
}

#[test]
fn test_ctor_uncertain_error() {
    assert_eq!(
        CausalityError::UncertainError("sampling failed").0,
        CausalityErrorEnum::UncertainError("sampling failed".to_string())
    );
}

#[test]
fn test_ctor_io_error() {
    assert_eq!(
        CausalityError::IoError("cannot read data.csv").0,
        CausalityErrorEnum::IoError("cannot read data.csv".to_string())
    );
}

#[test]
fn test_ctor_custom() {
    assert_eq!(
        CausalityError::Custom("observation is negative").0,
        CausalityErrorEnum::Custom("observation is negative".to_string())
    );
}

#[test]
fn test_ctor_action_error() {
    assert_eq!(
        CausalityError::ActionError("actuator offline").0,
        CausalityErrorEnum::ActionError("actuator offline".to_string())
    );
}

#[test]
fn test_ctor_deontic_error() {
    assert_eq!(
        CausalityError::DeonticError("norm graph is cyclic").0,
        CausalityErrorEnum::DeonticError("norm graph is cyclic".to_string())
    );
}

#[test]
fn test_ctor_model_error() {
    assert_eq!(
        CausalityError::ModelError("assumption failed").0,
        CausalityErrorEnum::ModelError("assumption failed".to_string())
    );
}

#[test]
fn test_string_constructors_accept_str_and_string() {
    let owned: String = format!("node {} missing", 9);
    assert_eq!(
        CausalityError::Custom(owned.clone()),
        CausalityError::Custom(owned.as_str())
    );
}

#[test]
fn test_constructors_build_pairwise_distinct_variants() {
    // Every constructor yields its own variant: no two collide, even with the same message.
    let msg = "same message";
    let all = [
        CausalityError::Unspecified(),
        CausalityError::InternalLogicError(msg),
        CausalityError::TypeConversionError(msg),
        CausalityError::ValueNotAvailable(),
        CausalityError::MissingParameter(msg),
        CausalityError::UnsupportedOperation(msg),
        CausalityError::MaxStepsExceeded(),
        CausalityError::UnexpectedCommand(msg),
        CausalityError::GraphNotFrozen(),
        CausalityError::GraphContainsCycle(),
        CausalityError::CausaloidNotFound(0),
        CausalityError::GraphError(msg),
        CausalityError::EmptyCollection(),
        CausalityError::MissingContext(),
        CausalityError::UncertainError(msg),
        CausalityError::IoError(msg),
        CausalityError::Custom(msg),
        CausalityError::ActionError(msg),
        CausalityError::DeonticError(msg),
        CausalityError::ModelError(msg),
    ];
    let distinct: HashSet<_> = all.iter().cloned().collect();
    assert_eq!(distinct.len(), all.len());
}
