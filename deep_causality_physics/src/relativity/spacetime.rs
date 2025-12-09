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
    // Ensure compatible Minkowski metric
    if t1.metric() != t2.metric() {
        return Err(PhysicsError::new(PhysicsErrorEnum::MetricSingularity(
            "Metric mismatch between vectors".into(),
        )));
    }
    if let deep_causality_multivector::Metric::Minkowski(_) = t1.metric() {
    } else {
        return Err(PhysicsError::new(PhysicsErrorEnum::MetricSingularity(
            "Time dilation requires Minkowski metric".into(),
        )));
    }

    // Inner product must yield a scalar
    let inner = t1.inner_product(t2);
    if inner.data().is_empty() {
        return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
            "Inner product did not yield any data".into(),
        )));
    }
    
    // Verify that non-scalar components (index > 0) are effectively zero
    // In dense representation (e.g. 16 dims), inner product of vectors should be scalar (index 0).
    let non_scalar_magnitude: f64 = inner.data().iter().skip(1).map(|v| v.abs()).sum();
    if non_scalar_magnitude > 1e-9 {
         return Err(PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
            format!("Inner product did not yield scalar grade (non-scalar mag: {})", non_scalar_magnitude).into(),
        )));
    }

    let dot = inner.data()[0];

    // Timelike check: squared magnitudes strictly positive in (+---)
    let s1 = t1.squared_magnitude();
    let s2 = t2.squared_magnitude();
    if !(s1 > 0.0 && s2 > 0.0) {
        return Err(PhysicsError::new(PhysicsErrorEnum::CausalityViolation(
            "Non-timelike vector encountered".into(),
        )));
    }
    let mag1 = s1.sqrt();
    let mag2 = s2.sqrt();

    let denom = mag1 * mag2;
    if denom == 0.0 || !denom.is_finite() {
        return Err(PhysicsError::new(PhysicsErrorEnum::NumericalInstability(
            "Invalid normalization in gamma computation".into(),
        )));
    }

    // Clamp gamma to handle floating-point noise
    let mut gamma = dot / denom;
    let eps = 1e-9;
    if gamma < 1.0 && (1.0 - gamma) <= eps {
        gamma = 1.0;
    }
    if gamma < 1.0 {
        return Err(PhysicsError::new(PhysicsErrorEnum::CausalityViolation(
            format!("Invalid Lorentz factor < 1.0: {}", gamma),
        )));
    }

    let eta = gamma.acosh();
    PhaseAngle::new(eta)
}

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
