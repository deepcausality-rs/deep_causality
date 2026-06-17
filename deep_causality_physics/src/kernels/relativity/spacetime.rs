/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::SpacetimeVector;

use crate::PhaseAngle;
use crate::error::PhysicsError;
use deep_causality_multivector::{CausalMultiVector, Metric, MultiVector};
use deep_causality_num::{Field, Float, RealField};

// Kernels

/// Calculates the spacetime interval $s^2 = g_{\mu\nu} x^\mu x^\nu$ (squared magnitude in metric).
///
/// # Arguments
/// * `x` - Spacetime vector.
/// * `metric` - Expected metric (checked against vector's internal metric).
///
/// # Returns
/// * `Ok(f64)` - Spacetime interval $s^2$.
pub fn spacetime_interval_kernel<R>(
    x: &CausalMultiVector<R>,
    metric: &Metric,
) -> Result<R, PhysicsError>
where
    R: RealField,
{
    // s^2 = g_uv x^u x^v
    // Validate that the vector's internal metric matches the context metric.
    // CausalMultiVector stores its immutable metric.
    if x.metric() != *metric {
        return Err(PhysicsError::MetricSingularity(format!(
            "Spacetime Interval Metric Mismatch: Vector has {:?}, Context expects {:?}",
            x.metric(),
            metric
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
pub fn time_dilation_angle_kernel<R>(
    t1: &CausalMultiVector<R>,
    t2: &CausalMultiVector<R>,
) -> Result<PhaseAngle<R>, PhysicsError>
where
    R: RealField,
{
    // Ensure compatible Minkowski metric
    if t1.metric() != t2.metric() {
        return Err(PhysicsError::MetricSingularity(
            "Metric mismatch between vectors".into(),
        ));
    }
    if let Metric::Minkowski(_) = t1.metric() {
    } else {
        return Err(PhysicsError::MetricSingularity(
            "Time dilation requires Minkowski metric".into(),
        ));
    }

    // Inner product must yield a scalar
    let inner = t1.inner_product(t2);
    if inner.data().is_empty() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Inner product did not yield any data".into(),
        ));
    }

    // Verify that non-scalar components (index > 0) are effectively zero
    // In dense representation (e.g. 16 dims), inner product of vectors should be scalar (index 0).
    let mut non_scalar_magnitude = R::zero();
    for v in inner.data().iter().skip(1) {
        non_scalar_magnitude += v.abs();
    }
    let dot = inner.data()[0];

    // Precision-aware grade-purity tolerance: scale by the scalar magnitude so
    // the test is meaningful at any precision (f32 / f64 / Float106). Using a
    // fixed absolute 1e-9 would under-tolerate near-c boosts at f32 and
    // over-tolerate at higher precisions.
    let eps_rel = R::epsilon().sqrt();
    // Scale by max(|dot|, 1) — fall back to 1 when |dot| is very small so the
    // tolerance never collapses to zero. RealField has no `max`, so do it
    // inline with a comparison.
    let dot_scale = if dot.abs() > R::one() {
        dot.abs()
    } else {
        R::one()
    };
    let grade_tol = eps_rel * dot_scale;
    if non_scalar_magnitude > grade_tol {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "Inner product did not yield scalar grade".into(),
        ));
    }

    // Timelike check: squared magnitudes strictly positive in (+---)
    let s1 = t1.squared_magnitude();
    let s2 = t2.squared_magnitude();
    let zero = R::zero();
    if !(s1 > zero && s2 > zero) {
        return Err(PhysicsError::CausalityViolation(
            "Non-timelike vector encountered".into(),
        ));
    }
    let mag1 = s1.sqrt();
    let mag2 = s2.sqrt();

    let denom = mag1 * mag2;
    if denom == zero || !denom.is_finite() {
        return Err(PhysicsError::NumericalInstability(
            "Invalid normalization in gamma computation".into(),
        ));
    }

    // Clamp gamma to handle floating-point noise near 1.0. The tolerance is
    // sqrt(epsilon) — the natural first-order precision around unity — so the
    // clamp adapts to f32 / f64 / Float106 without false-failing near-1 gammas
    // at lower precisions.
    let one = R::one();
    let mut gamma = dot / denom;
    if gamma < one && (one - gamma) <= eps_rel {
        gamma = one;
    }
    if gamma < one {
        return Err(PhysicsError::CausalityViolation(
            "Invalid Lorentz factor < 1.0".into(),
        ));
    }

    // acosh(gamma) = ln(gamma + sqrt(gamma^2 - 1))
    let eta = (gamma + (gamma * gamma - one).sqrt()).ln();
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
pub fn chronometric_volume_kernel<R>(
    a: &CausalMultiVector<R>,
    b: &CausalMultiVector<R>,
    c: &CausalMultiVector<R>,
) -> Result<SpacetimeVector<R>, PhysicsError>
where
    R: RealField,
{
    // Volume formed by trivector a ^ b ^ c
    // V = a ^ b ^ c

    // Ensure all inputs share the same metric
    if a.metric() != b.metric() || a.metric() != c.metric() {
        return Err(PhysicsError::MetricSingularity(
            "chronometric_volume requires all vectors to share the same metric".into(),
        ));
    }

    let v = a.outer_product(b).outer_product(c);
    Ok(SpacetimeVector(v))
}

/// Generates a Schwarzschild-like 4D metric tensor.
///
/// Signature: (- + + +) standard GR convention.
/// Returns a diagonal metric tensor.
///
/// # Arguments
/// * `g_00` - Time dilation component (usually $-(1 - r_s/r)$).
/// * `g_11` - Radial stretching component (usually $(1 - r_s/r)^{-1}$).
/// * `g_22` - Angular component (e.g., $r^2$).
/// * `g_33` - Angular component (e.g., $r^2 \sin^2\theta$).
///
/// # Returns
/// * `Result<CausalTensor<T>, PhysicsError>` - The metric tensor.
pub fn generate_schwarzschild_metric<T>(
    g_00: T,
    g_11: T,
    g_22: T,
    g_33: T,
) -> Result<deep_causality_tensor::CausalTensor<T>, PhysicsError>
where
    T: Field + Float,
{
    let metric_data = vec![
        g_00,
        T::zero(),
        T::zero(),
        T::zero(),
        T::zero(),
        g_11,
        T::zero(),
        T::zero(),
        T::zero(),
        T::zero(),
        g_22,
        T::zero(),
        T::zero(),
        T::zero(),
        T::zero(),
        g_33,
    ];

    deep_causality_tensor::CausalTensor::new(metric_data, vec![4, 4]).map_err(PhysicsError::from)
}

/// Parallel transports a vector along a discrete path using Christoffel symbols.
///
/// Solves: $\frac{Dv^\mu}{d\lambda} = \frac{dv^\mu}{d\lambda} + \Gamma^\mu_{\nu\rho} \frac{dx^\nu}{d\lambda} v^\rho = 0$
///
/// Uses Euler method for simplicity along discretized path segments.
///
/// # Arguments
/// * `initial_vector` - Vector $v_0^\mu$ to transport.
/// * `path` - List of positions along the path (each position is a slice of coordinates).
/// * `christoffel` - Christoffel symbols $\Gamma^\mu_{\nu\rho}$ (Rank 3 tensor [dim, dim, dim]).
///
/// # Returns
/// * `Ok(Vec<T>)` - The parallel-transported vector at the end of the path.
pub fn parallel_transport_kernel<T>(
    initial_vector: &[T],
    path: &[Vec<T>],
    christoffel: &deep_causality_tensor::CausalTensor<T>,
) -> Result<Vec<T>, PhysicsError>
where
    T: Field + Float + From<f64> + Copy,
{
    if path.len() < 2 {
        return Err(PhysicsError::DimensionMismatch(
            "Path must have at least 2 points".into(),
        ));
    }

    let dim = initial_vector.len();
    if christoffel.num_dim() != 3 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Christoffel symbols must be rank 3, got {}",
            christoffel.num_dim()
        )));
    }

    let shape = christoffel.shape();
    if shape[0] != dim || shape[1] != dim || shape[2] != dim {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Christoffel shape {:?} incompatible with dimension {}",
            shape, dim
        )));
    }

    // Validate path dimensions
    for (i, point) in path.iter().enumerate() {
        if point.len() != dim {
            return Err(PhysicsError::DimensionMismatch(format!(
                "Path point {} has dimension {}, expected {}",
                i,
                point.len(),
                dim
            )));
        }
    }

    let christoffel_data = christoffel.as_slice();
    let mut v = initial_vector.to_vec();

    // Transport along each segment
    for i in 0..path.len() - 1 {
        // Compute tangent vector dx^nu (difference between consecutive points)
        let dx: Vec<T> = path[i + 1]
            .iter()
            .zip(path[i].iter())
            .map(|(x1, x0)| *x1 - *x0)
            .collect();

        // Update: dv^mu = -Gamma^mu_nu_rho * dx^nu * v^rho
        let mut dv = vec![T::zero(); dim];
        for mu in 0..dim {
            for nu in 0..dim {
                for rho in 0..dim {
                    let gamma = christoffel_data[mu * dim * dim + nu * dim + rho];
                    dv[mu] -= gamma * dx[nu] * v[rho];
                }
            }
        }

        // Euler step: v_new = v + dv
        for mu in 0..dim {
            v[mu] += dv[mu];
        }

        // Check for numerical instability
        if v.iter().any(|x| !x.is_finite()) {
            return Err(PhysicsError::NumericalInstability(format!(
                "Parallel transport diverged at segment {}",
                i
            )));
        }
    }

    Ok(v)
}

/// Computes the proper time along a discrete worldline.
///
/// $\tau = \int \sqrt{-g_{\mu\nu} \frac{dx^\mu}{d\lambda} \frac{dx^\nu}{d\lambda}} \, d\lambda$
///
/// For a discrete path, sums the proper time increments between consecutive points.
///
/// # Arguments
/// * `path` - List of spacetime positions along the worldline.
/// * `metric` - Metric tensor $g_{\mu\nu}$ (Rank 2 tensor [dim, dim]).
///
/// # Returns
/// * `Ok(T)` - Total proper time along the path.
pub fn proper_time_kernel<T>(
    path: &[Vec<T>],
    metric: &deep_causality_tensor::CausalTensor<T>,
) -> Result<T, PhysicsError>
where
    T: Field + Float + From<f64> + Copy,
{
    if path.len() < 2 {
        return Ok(T::zero()); // No proper time for single point or empty path
    }

    if metric.num_dim() != 2 {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric must be rank 2, got {}",
            metric.num_dim()
        )));
    }

    let shape = metric.shape();
    let dim = shape[0];
    if shape[1] != dim {
        return Err(PhysicsError::DimensionMismatch(format!(
            "Metric must be square, got {:?}",
            shape
        )));
    }

    // Validate path dimensions
    for (i, point) in path.iter().enumerate() {
        if point.len() != dim {
            return Err(PhysicsError::DimensionMismatch(format!(
                "Path point {} has dimension {}, expected {}",
                i,
                point.len(),
                dim
            )));
        }
    }

    let metric_data = metric.as_slice();
    let mut total_tau = T::zero();

    for i in 0..path.len() - 1 {
        // Compute displacement dx^mu
        let dx: Vec<T> = path[i + 1]
            .iter()
            .zip(path[i].iter())
            .map(|(x1, x0)| *x1 - *x0)
            .collect();

        // Compute ds^2 = g_mu_nu dx^mu dx^nu
        let mut ds_squared = T::zero();
        for mu in 0..dim {
            for nu in 0..dim {
                let g_munu = metric_data[mu * dim + nu];
                ds_squared += g_munu * dx[mu] * dx[nu];
            }
        }

        // For timelike intervals: ds^2 < 0 (East Coast) or > 0 (West Coast)
        // Proper time: d\tau = sqrt(|ds^2|) for timelike
        // We take the absolute value to handle both conventions
        let dtau = ds_squared.abs().sqrt();

        if !dtau.is_finite() {
            return Err(PhysicsError::NumericalInstability(format!(
                "Non-finite proper time increment at segment {}",
                i
            )));
        }

        total_tau += dtau;
    }

    Ok(total_tau)
}
