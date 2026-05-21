/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::quantum::quantities::PhaseAngle;
use crate::relativity::gravity;
use crate::relativity::quantities::SpacetimeVector;
use crate::relativity::spacetime;
use core::fmt::Debug;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;

/// Causal wrapper for [`gravity::einstein_tensor_kernel`].
pub fn einstein_tensor(
    ricci: &CausalTensor<f64>,
    scalar_r: f64,
    metric: &CausalTensor<f64>,
) -> PropagatingEffect<CausalTensor<f64>> {
    match gravity::einstein_tensor_kernel(ricci, scalar_r, metric) {
        Ok(g) => PropagatingEffect::pure(g),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`gravity::geodesic_deviation_kernel`].
///
/// Computes the geodesic deviation acceleration: A^μ = -R^μ_νσρ V^ν n^σ V^ρ
pub fn geodesic_deviation(
    riemann: &CausalTensor<f64>,
    velocity: &[f64],
    separation: &[f64],
) -> PropagatingEffect<Vec<f64>> {
    match gravity::geodesic_deviation_kernel(riemann, velocity, separation) {
        Ok(acc) => PropagatingEffect::pure(acc),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`spacetime::spacetime_interval_kernel`].
pub fn spacetime_interval<R>(x: &CausalMultiVector<R>, metric: &Metric) -> PropagatingEffect<R>
where
    R: RealField + Default + Debug,
{
    match spacetime::spacetime_interval_kernel(x, metric) {
        Ok(s2) => PropagatingEffect::pure(s2),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`spacetime::time_dilation_angle_kernel`].
pub fn time_dilation_angle<R>(
    t1: &CausalMultiVector<R>,
    t2: &CausalMultiVector<R>,
) -> PropagatingEffect<PhaseAngle<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match spacetime::time_dilation_angle_kernel(t1, t2) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

/// Causal wrapper for [`spacetime::chronometric_volume_kernel`].
pub fn chronometric_volume<R>(
    a: &CausalMultiVector<R>,
    b: &CausalMultiVector<R>,
    c: &CausalMultiVector<R>,
) -> PropagatingEffect<SpacetimeVector<R>>
where
    R: RealField + Debug,
{
    match spacetime::chronometric_volume_kernel(a, b, c) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
