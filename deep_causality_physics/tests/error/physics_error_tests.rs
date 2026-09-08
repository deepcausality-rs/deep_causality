/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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

#[test]
fn test_metric_convention_error() {
    // Exercises the MetricConventionError constructor (physics_error.rs:104-106).
    let msg = "convention mismatch".to_string();
    let err = PhysicsError::MetricConventionError(msg.clone());
    match err.0 {
        PhysicsErrorEnum::MetricConventionError(m) => assert_eq!(m, msg),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_topology_error() {
    let msg = "topology problem".to_string();
    let err = PhysicsError::TopologyError(msg.clone());
    match err.0 {
        PhysicsErrorEnum::TopologyError(m) => assert_eq!(m, msg),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_metric_convention_error_display() {
    // Exercises the MetricConventionError Display arm (physics_error.rs:156-157).
    assert_eq!(
        format!("{}", PhysicsError::MetricConventionError("oops".into())),
        "Metric Convention Error: oops"
    );
}

#[test]
fn test_topology_error_display() {
    assert_eq!(
        format!("{}", PhysicsError::TopologyError("graph".into())),
        "Topology Error: graph"
    );
}

#[test]
fn test_from_metric_error() {
    // Exercises From<MetricError> for PhysicsError (physics_error.rs:132-134).
    // Construct a real MetricError and convert it.
    let metric_err = deep_causality_metric::MetricError::ValidationFailed("bad metric".into());
    let err: PhysicsError = PhysicsError::from(metric_err);
    match err.0 {
        PhysicsErrorEnum::MetricConventionError(m) => assert!(!m.is_empty()),
        other => panic!("Wrong variant: {:?}", other),
    }
}

#[test]
fn test_from_stats_error_keeps_not_converged_as_not_converged() {
    // `StatsErrorEnum::NotConverged` and `PhysicsErrorEnum::NotConverged` draw the same line: an
    // iterative fit that ran out of iterations is retryable at a wider cap, an unstable
    // computation is not. The catch-all arm used to collapse the first into the second, which
    // erased the only classification a caller can act on.
    let err: PhysicsError =
        PhysicsError::from(deep_causality_stats::StatsError::NotConverged(25, "IRLS"));
    match err.0 {
        PhysicsErrorEnum::NotConverged(m) => {
            // The stats `Display` carries the iteration count, and it must survive the crossing.
            assert!(m.contains("25"), "the iteration count was dropped: {m}");
            assert!(m.contains("IRLS"), "the detail was dropped: {m}");
        }
        other => panic!("Wrong variant: {:?}", other),
    }
}

#[test]
fn test_from_stats_error_maps_the_other_classes_unchanged() {
    // The arms that were already right, pinned so the new one cannot be widened by accident.
    use deep_causality_stats::StatsError;

    let shape: PhysicsError = PhysicsError::from(StatsError::DimensionMismatch("x vs y"));
    assert!(matches!(shape.0, PhysicsErrorEnum::DimensionMismatch(_)));

    let empty: PhysicsError = PhysicsError::from(StatsError::EmptyInput("no rows"));
    assert!(matches!(empty.0, PhysicsErrorEnum::DimensionMismatch(_)));

    let prob: PhysicsError = PhysicsError::from(StatsError::NegativeProbability("p < 0"));
    assert!(matches!(prob.0, PhysicsErrorEnum::NormalizationError(_)));

    // Still the catch-all: a rank-deficient design is not a cap that can be raised.
    let rank: PhysicsError = PhysicsError::from(StatsError::RankDeficient("singular design"));
    assert!(matches!(rank.0, PhysicsErrorEnum::NumericalInstability(_)));
}
