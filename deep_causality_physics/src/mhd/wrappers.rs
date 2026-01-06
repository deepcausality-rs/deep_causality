/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::mhd::quantities::{
    AlfvenSpeed, DebyeLength, Diffusivity, LarmorRadius, MagneticPressure,
};
use crate::mhd::{grmhd, ideal, plasma, resistive};
use crate::{Density, Mass, PhysicalField, Speed, Temperature};
use deep_causality_core::{CausalityError, PropagatingEffect};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::Manifold;

// ============================================================================
// Ideal MHD Wrappers
// ============================================================================

pub fn alfven_speed(b: &PhysicalField, rho: &Density, mu0: f64) -> PropagatingEffect<AlfvenSpeed> {
    match ideal::alfven_speed_kernel(b, rho, mu0) {
        Ok(v) => PropagatingEffect::pure(v),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn magnetic_pressure(b: &PhysicalField, mu0: f64) -> PropagatingEffect<MagneticPressure> {
    match ideal::magnetic_pressure_kernel(b, mu0) {
        Ok(p) => PropagatingEffect::pure(p),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn ideal_induction(
    v: &Manifold<f64, f64>,
    b: &Manifold<f64, f64>,
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
    b: &Manifold<f64, f64>,
    eta: Diffusivity,
) -> PropagatingEffect<CausalTensor<f64>> {
    match resistive::resistive_diffusion_kernel(b, eta) {
        Ok(t) => PropagatingEffect::pure(t),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn magnetic_reconnection_rate(va: AlfvenSpeed, s: f64) -> PropagatingEffect<Speed> {
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
    em_manifold: &Manifold<f64, f64>,
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

pub fn debye_length(t: Temperature, n: f64, eps0: f64, e: f64) -> PropagatingEffect<DebyeLength> {
    match plasma::debye_length_kernel(t, n, eps0, e) {
        Ok(l) => PropagatingEffect::pure(l),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}

pub fn larmor_radius(
    m: Mass,
    v: Speed,
    q: f64,
    b: &PhysicalField,
) -> PropagatingEffect<LarmorRadius> {
    match plasma::larmor_radius_kernel(m, v, q, b) {
        Ok(r) => PropagatingEffect::pure(r),
        Err(e) => PropagatingEffect::from_error(CausalityError::from(e)),
    }
}
