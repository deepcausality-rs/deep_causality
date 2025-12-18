/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalityError;
use deep_causality_physics::{PhysicsError, PhysicsErrorEnum};
use deep_causality_tensor::CausalTensorError;

#[test]
fn test_physical_invariant_broken() {
    let msg = "test error".to_string();
    let err = PhysicsError::PhysicalInvariantBroken(msg.clone());
    match err.0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(m) => assert_eq!(m, msg),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_dimension_mismatch() {
    let msg = "test error".to_string();
    let err = PhysicsError::DimensionMismatch(msg.clone());
    match err.0 {
        PhysicsErrorEnum::DimensionMismatch(m) => assert_eq!(m, msg),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_causality_violation() {
    let msg = "test error".to_string();
    let err = PhysicsError::CausalityViolation(msg.clone());
    match err.0 {
        PhysicsErrorEnum::CausalityViolation(m) => assert_eq!(m, msg),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_metric_singularity() {
    let msg = "test error".to_string();
    let err = PhysicsError::MetricSingularity(msg.clone());
    match err.0 {
        PhysicsErrorEnum::MetricSingularity(m) => assert_eq!(m, msg),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_normalization_error() {
    let msg = "test error".to_string();
    let err = PhysicsError::NormalizationError(msg.clone());
    match err.0 {
        PhysicsErrorEnum::NormalizationError(m) => assert_eq!(m, msg),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_zero_kelvin_violation() {
    let err = PhysicsError::ZeroKelvinViolation();
    assert!(matches!(err.0, PhysicsErrorEnum::ZeroKelvinViolation));
}

#[test]
fn test_entropy_violation() {
    let msg = "test error".to_string();
    let err = PhysicsError::EntropyViolation(msg.clone());
    match err.0 {
        PhysicsErrorEnum::EntropyViolation(m) => assert_eq!(m, msg),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_singularity() {
    let msg = "test error".to_string();
    let err = PhysicsError::Singularity(msg.clone());
    match err.0 {
        PhysicsErrorEnum::Singularity(m) => assert_eq!(m, msg),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_numerical_instability() {
    let msg = "test error".to_string();
    let err = PhysicsError::NumericalInstability(msg.clone());
    match err.0 {
        PhysicsErrorEnum::NumericalInstability(m) => assert_eq!(m, msg),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_calculation_error() {
    let msg = "test error".to_string();
    let err = PhysicsError::CalculationError(msg.clone());
    match err.0 {
        PhysicsErrorEnum::CalculationError(m) => assert_eq!(m, msg),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_display() {
    assert_eq!(
        format!("{}", PhysicsError::PhysicalInvariantBroken("msg".into())),
        "Physical Invariant Broken: msg"
    );
    assert_eq!(
        format!("{}", PhysicsError::DimensionMismatch("msg".into())),
        "Dimension Mismatch: msg"
    );
    assert_eq!(
        format!("{}", PhysicsError::CausalityViolation("msg".into())),
        "Causality Violation: msg"
    );
    assert_eq!(
        format!("{}", PhysicsError::MetricSingularity("msg".into())),
        "Metric Singularity: msg"
    );
    assert_eq!(
        format!("{}", PhysicsError::NormalizationError("msg".into())),
        "Normalization Error: msg"
    );
    assert_eq!(
        format!("{}", PhysicsError::ZeroKelvinViolation()),
        "Zero Kelvin Violation: Temperature cannot be negative"
    );
    assert_eq!(
        format!("{}", PhysicsError::EntropyViolation("msg".into())),
        "Entropy Violation: msg"
    );
    assert_eq!(
        format!("{}", PhysicsError::Singularity("msg".into())),
        "Singularity: msg"
    );
    assert_eq!(
        format!("{}", PhysicsError::NumericalInstability("msg".into())),
        "Numerical Instability: msg"
    );
    assert_eq!(
        format!("{}", PhysicsError::CalculationError("msg".into())),
        "Calculation Error: msg"
    );
}

#[test]
fn test_debug() {
    let err = PhysicsError::ZeroKelvinViolation();
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("PhysicsError"));
    assert!(debug_str.contains("ZeroKelvinViolation"));
}

#[test]
fn test_from_causal_tensor_error() {
    let tensor_err = CausalTensorError::ShapeMismatch;
    let err: PhysicsError = PhysicsError::from(tensor_err);
    match err.0 {
        PhysicsErrorEnum::Singularity(m) => assert!(m.contains("Tensor Error")),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_into_causality_error() {
    let err = PhysicsError::MetricSingularity("test".into());
    let causality_err: CausalityError = err.into();
    let err_str = format!("{}", causality_err);
    assert!(err_str.contains("Metric Singularity: test"));
}
