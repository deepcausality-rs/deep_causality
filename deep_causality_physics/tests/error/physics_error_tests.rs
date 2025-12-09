/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::CausalityError;
use deep_causality_physics::{PhysicsError, PhysicsErrorEnum};

#[test]
fn test_physics_error_variants_display() {
    let variants = vec![
        (
            PhysicsErrorEnum::PhysicalInvariantBroken("Mass < 0".into()),
            "Physical Invariant Broken: Mass < 0",
        ),
        (
            PhysicsErrorEnum::DimensionMismatch("Rank 2 vs 3".into()),
            "Dimension Mismatch: Rank 2 vs 3",
        ),
        (
            PhysicsErrorEnum::CausalityViolation("Spacelike".into()),
            "Causality Violation: Spacelike",
        ),
        (
            PhysicsErrorEnum::MetricSingularity("Det = 0".into()),
            "Metric Singularity: Det = 0",
        ),
        (
            PhysicsErrorEnum::NormalizationError("Sum != 1".into()),
            "Normalization Error: Sum != 1",
        ),
        (
            PhysicsErrorEnum::ZeroKelvinViolation,
            "Zero Kelvin Violation: Temperature cannot be negative",
        ),
        (
            PhysicsErrorEnum::EntropyViolation("dS < 0".into()),
            "Entropy Violation: dS < 0",
        ),
        (
            PhysicsErrorEnum::Singularity("Div by zero".into()),
            "Singularity: Div by zero",
        ),
        (
            PhysicsErrorEnum::NumericalInstability("NaN detected".into()),
            "Numerical Instability: NaN detected",
        ),
        (
            PhysicsErrorEnum::CalculationError("Matrix inverse failed".into()),
            "Calculation Error: Matrix inverse failed",
        ),
    ];

    for (variant, expected_msg) in variants {
        let err = PhysicsError::new(variant);
        let msg = format!("{}", err);
        assert_eq!(msg, expected_msg, "Failed for variant {:?}", err);
    }
}

#[test]
fn test_physics_error_conversion_to_causality_error() {
    // Test conversion specifically for one variant, as the mapping logic is generic (ToString)
    let p_err = PhysicsError::new(PhysicsErrorEnum::ZeroKelvinViolation);
    let c_err: CausalityError = p_err.into();
    let msg = format!("{}", c_err);
    assert!(msg.contains("Zero Kelvin Violation"));
}
