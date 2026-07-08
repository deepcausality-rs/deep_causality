/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::kernels::mhd::{grmhd, ideal, plasma, resistive};
use crate::{
    AlfvenSpeed, DebyeLength, Diffusivity, ElectronDensity, LarmorRadius, MagneticPressure,
    PlasmaFrequency,
};
use crate::{Density, Mass, PhysicalField, Speed, Temperature};
use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_num::FromPrimitive;
use deep_causality_par::MaybeParallel;
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
    R: RealField + MaybeParallel + Debug,
{
    match ideal::alfven_speed_kernel(b, rho, mu0) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn magnetic_pressure<R>(b: &PhysicalField<R>, mu0: R) -> PropagatingEffect<MagneticPressure<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match ideal::magnetic_pressure_kernel(b, mu0) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn ideal_induction<R>(
    v: &SimplicialManifold<R, R>,
    b: &SimplicialManifold<R, R>,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
{
    match ideal::ideal_induction_kernel(v, b) {
        Ok(t) => PropagatingEffect::pure(t),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// Resistive MHD Wrappers
// ============================================================================

pub fn resistive_diffusion<R>(
    b: &SimplicialManifold<R, R>,
    eta: Diffusivity<R>,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
{
    match resistive::resistive_diffusion_kernel(b, eta) {
        Ok(t) => PropagatingEffect::pure(t),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn magnetic_reconnection_rate<R>(va: AlfvenSpeed<R>, s: R) -> PropagatingEffect<Speed<R>>
where
    R: RealField + MaybeParallel + Debug,
{
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
pub fn relativistic_current<R, M>(
    em_manifold: &SimplicialManifold<R, R>,
    spacetime_metric: &M,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Default + PartialEq + Debug,
    M: LorentzianMetric,
{
    match grmhd::relativistic_current_kernel(em_manifold, spacetime_metric) {
        Ok(j) => PropagatingEffect::pure(j),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn energy_momentum_tensor_em<R>(
    em: &CausalTensor<R>,
    metric: &CausalTensor<R>,
) -> PropagatingEffect<CausalTensor<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + core::iter::Sum + Default + PartialOrd + Debug,
{
    match grmhd::energy_momentum_tensor_em_kernel(em, metric) {
        Ok(t) => PropagatingEffect::pure(t),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

// ============================================================================
// Plasma Wrappers
// ============================================================================

pub fn debye_length<R>(t: Temperature<R>, n: R, eps0: R, e: R) -> PropagatingEffect<DebyeLength<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match plasma::debye_length_kernel(t, n, eps0, e) {
        Ok(l) => PropagatingEffect::pure(l),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn larmor_radius<R>(
    m: Mass<R>,
    v: Speed<R>,
    q: R,
    b: &PhysicalField<R>,
) -> PropagatingEffect<LarmorRadius<R>>
where
    R: RealField + MaybeParallel + Debug,
{
    match plasma::larmor_radius_kernel(m, v, q, b) {
        Ok(r) => PropagatingEffect::pure(r),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn plasma_frequency<R>(n_e: ElectronDensity<R>) -> PropagatingEffect<PlasmaFrequency<R>>
where
    R: RealField + MaybeParallel + FromPrimitive + Debug,
{
    match plasma::plasma_frequency_kernel(n_e) {
        Ok(w) => PropagatingEffect::pure(w),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
