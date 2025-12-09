/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::relativity::quantities::SpacetimeVector;

use crate::error::{PhysicsError, PhysicsErrorEnum};
use crate::quantum::quantities::PhaseAngle;
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};

// Kernels

/// Calculates the spacetime interval $s^2 = g_{\mu\nu} x^\mu x^\nu$ (squared magnitude in metric).
///
/// # Arguments
/// * `x` - Spacetime vector.
/// * `metric` - Expected metric (checked against vector's internal metric).
///
/// # Returns
/// * `Ok(f64)` - Spacetime interval $s^2$.
pub fn spacetime_interval_kernel(
    x: &CausalMultiVector<f64>,
    metric: &Metric,
) -> Result<f64, PhysicsError> {
    // s^2 = g_uv x^u x^v
    // Validate that the vector's internal metric matches the context metric.
    // CausalMultiVector stores its immutable metric.
    if x.metric() != *metric {
        return Err(PhysicsError::new(PhysicsErrorEnum::MetricSingularity(
            format!(
                "Spacetime Interval Metric Mismatch: Vector has {:?}, Context expects {:?}",
                x.metric(),
                metric
            ),
        )));
    }

    Ok(x.squared_magnitude())
}

/// Calculates Time Dilation Angle (Rapidity $\eta$).
///
/// $\cosh(\eta) = \gamma = \frac{t_1 \cdot t_2}{|t_1| |t_2|}$.
///
/// Use `PhaseAngle` here to represent the hyperbolic angle.
///
/// # Arguments
/// * `t1` - Spacetime vector 1 (Timelike).
/// * `t2` - Spacetime vector 2 (Timelike).
///
/// # Returns
/// * `Result<PhaseAngle, PhysicsError>` - Rapidity $\eta$.
pub fn time_dilation_angle_kernel(
    t1: &CausalMultiVector<f64>,
    t2: &CausalMultiVector<f64>,
) -> Result<PhaseAngle, PhysicsError> {
    // Hyperbolic angle (Rapidity, eta) between two timelike vectors.
    // cosh(eta) = (t1 . t2) / (|t1| |t2|) = gamma
    // eta = acosh(gamma)

    // 1. Calculate Dot Product
    let dot = t1.inner_product(t2).data()[0]; // Scalar

    // 2. Calculate Magnitudes
    let mag1 = t1.squared_magnitude().sqrt();
    let mag2 = t2.squared_magnitude().sqrt();

    if mag1 == 0.0 || mag2 == 0.0 {
        return Err(PhysicsError::new(
            PhysicsErrorEnum::PhysicalInvariantBroken(
                "Zero magnitude vector in time dilation calculation".into(),
            ),
        ));
    }

    // 3. Calculate Gamma (Lorentz Factor)
    // For timelike vectors in (+---) metric, t^2 > 0.
    // inner product should be positive for future-pointing timelike vectors.
    let gamma = dot / (mag1 * mag2);

    // 4. Calculate Rapidity eta
    // Gamma must be >= 1.0 for valid acosh.
    // Floating point errors might produce 0.999999.
    if gamma < 1.0 {
        // Check if close to 1.0 (parallel vectors)
        if (gamma - 1.0).abs() < 1e-6 {
            return PhaseAngle::new(0.0);
        }
        // If significantly less than 1, physical invariant broken (not timelike/causal relation)
        return Err(PhysicsError::new(PhysicsErrorEnum::CausalityViolation(
            format!("Invalid Lorentz factor < 1.0: {}", gamma),
        )));
    }

    let eta = gamma.acosh();

    PhaseAngle::new(eta)}

/// Calculates Chronometric Volume (4-Volume) from 3 vectors?
///
/// NOTE: The implementation takes 3 vectors ($a, b, c$), effectively calculating a 3-Volume hyper-surface in Spacetime?
/// Or if input vectors are 4D, $a \wedge b \wedge c$ is a trivector.
///
/// # Arguments
/// * `a` - 1st Vector.
/// * `b` - 2nd Vector.
/// * `c` - 3rd Vector.
///
/// # Returns
/// * `Result<SpacetimeVector, PhysicsError>` - Trivector result (wrapped).
pub fn chronometric_volume_kernel(
    a: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
    c: &CausalMultiVector<f64>,
) -> Result<SpacetimeVector, PhysicsError> {
    // Volume formed by trivector a ^ b ^ c
    // V = a ^ b ^ c
    let v = a.outer_product(b).outer_product(c);
    Ok(SpacetimeVector(v))
}
