/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::mhd::quantities::{
    AlfvenSpeed, DebyeLength, Diffusivity, LarmorRadius, MagneticPressure,
};
use crate::mhd::{grmhd, ideal, plasma, resistive};
use crate::{Density, Mass, PhysicalField, Speed, Temperature};
use core::fmt::Debug;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::{FromPrimitive, RealField};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::SimplicialManifold;

// ============================================================================
// Ideal MHD Wrappers
// ============================================================================

pub fn alfven_speed<R>(
    b: &PhysicalField<R>,
    rho: &Density<R>,
    mu0: R,
) -> PropagatingEffect<AlfvenSpeed<R>>
where
    R: RealField + Debug,
{
    match ideal::alfven_speed_kernel(b, rho, mu0) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn magnetic_pressure<R>(b: &PhysicalField<R>, mu0: R) -> PropagatingEffect<MagneticPressure<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match ideal::magnetic_pressure_kernel(b, mu0) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn ideal_induction(
    v: &SimplicialManifold<f64, f64>,
    b: &SimplicialManifold<f64, f64>,
) -> PropagatingEffect<CausalTensor<f64>> {
    match ideal::ideal_induction_kernel(v, b) {
        Ok(t) => PropagatingEffect::pure(t),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// Resistive MHD Wrappers
// ============================================================================

pub fn resistive_diffusion(
    b: &SimplicialManifold<f64, f64>,
    eta: Diffusivity<f64>,
) -> PropagatingEffect<CausalTensor<f64>> {
    match resistive::resistive_diffusion_kernel(b, eta) {
        Ok(t) => PropagatingEffect::pure(t),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn magnetic_reconnection_rate(va: AlfvenSpeed<f64>, s: f64) -> PropagatingEffect<Speed> {
    match resistive::magnetic_reconnection_rate_kernel(va, s) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// GRMHD Wrappers
// ============================================================================

use crate::LorentzianMetric;

/// Wrapper for relativistic current density calculation.
///
/// Computes J = ★d★F using differential forms on the manifold.
pub fn relativistic_current<M: LorentzianMetric>(
    em_manifold: &SimplicialManifold<f64, f64>,
    spacetime_metric: &M,
) -> PropagatingEffect<CausalTensor<f64>> {
    match grmhd::relativistic_current_kernel(em_manifold, spacetime_metric) {
        Ok(j) => PropagatingEffect::pure(j),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn energy_momentum_tensor_em(
    em: &CausalTensor<f64>,
    metric: &CausalTensor<f64>,
) -> PropagatingEffect<CausalTensor<f64>> {
    match grmhd::energy_momentum_tensor_em_kernel(em, metric) {
        Ok(t) => PropagatingEffect::pure(t),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// Plasma Wrappers
// ============================================================================

pub fn debye_length<R>(t: Temperature, n: R, eps0: R, e: R) -> PropagatingEffect<DebyeLength<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match plasma::debye_length_kernel(t, n, eps0, e) {
        Ok(l) => PropagatingEffect::pure(l),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn larmor_radius<R>(
    m: Mass,
    v: Speed,
    q: R,
    b: &PhysicalField<R>,
) -> PropagatingEffect<LarmorRadius<R>>
where
    R: RealField + FromPrimitive + Debug,
{
    match plasma::larmor_radius_kernel(m, v, q, b) {
        Ok(r) => PropagatingEffect::pure(r),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
