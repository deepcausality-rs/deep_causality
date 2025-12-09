/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::relativity::quantities::SpacetimeVector;

use crate::error::{PhysicsError, PhysicsErrorEnum};
use crate::quantum::quantities::PhaseAngle;
use deep_causality_core::{CausalityError, PropagatingEffect};
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

// Wrappers

/// Causal wrapper for [`spacetime_interval_kernel`].
pub fn spacetime_interval(x: &CausalMultiVector<f64>, metric: &Metric) -> PropagatingEffect<f64> {
    match spacetime_interval_kernel(x, metric) {
        Ok(s2) => PropagatingEffect::pure(s2),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
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
/// * `PropagatingEffect<PhaseAngle>` - Rapidity $\eta$.
pub fn time_dilation_angle(
    t1: &CausalMultiVector<f64>,
    t2: &CausalMultiVector<f64>,
) -> PropagatingEffect<PhaseAngle> {
    // Hyperbolic angle (Rapidity, eta) between two timelike vectors.
    // cosh(eta) = (t1 . t2) / (|t1| |t2|) = gamma
    // eta = acosh(gamma)

    // 1. Calculate Dot Product
    let dot = t1.inner_product(t2).data()[0]; // Scalar

    // 2. Calculate Magnitudes
    let mag1 = t1.squared_magnitude().sqrt();
    let mag2 = t2.squared_magnitude().sqrt();

    if mag1 == 0.0 || mag2 == 0.0 {
        return PropagatingEffect::from_error(
            PhysicsError::new(PhysicsErrorEnum::PhysicalInvariantBroken(
                "Zero magnitude vector in time dilation calculation".into(),
            ))
            .into(),
        );
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
            return match PhaseAngle::new(0.0) {
                Ok(p) => PropagatingEffect::pure(p),
                Err(e) => PropagatingEffect::from_error(e.into()),
            };
        }
        // If significantly less than 1, physical invariant broken (not timelike/causal relation)
        return PropagatingEffect::from_error(
            PhysicsError::new(PhysicsErrorEnum::CausalityViolation(format!(
                "Invalid Lorentz factor < 1.0: {}",
                gamma
            )))
            .into(),
        );
    }

    let eta = gamma.acosh();

    match PhaseAngle::new(eta) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(e.into()),
    }
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
/// * `PropagatingEffect<SpacetimeVector>` - Trivector result (wrapped).
pub fn chronometric_volume(
    a: &CausalMultiVector<f64>,
    b: &CausalMultiVector<f64>,
    c: &CausalMultiVector<f64>,
) -> PropagatingEffect<SpacetimeVector> {
    // Volume formed by trivector a ^ b ^ c
    // V = a ^ b ^ c
    let v = a.outer_product(b).outer_product(c);
    PropagatingEffect::pure(SpacetimeVector(v))
}
