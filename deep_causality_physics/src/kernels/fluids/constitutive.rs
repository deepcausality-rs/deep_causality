/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Constitutive kernels for fluid viscous stress.
//!
//! Sign convention: stress positive in tension (continuum-mechanics convention).
//! The Newtonian viscous stress for an incompressible fluid is
//! `τ_ij = 2μ S_ij`. For a compressible Newtonian fluid with Stokes hypothesis
//! (bulk viscosity ζ = 0) it is `τ_ij = 2μ S_ij − (2/3)μ (∇·u) δ_ij`. With
//! non-zero bulk viscosity it is `τ_ij = 2μ S_ij − (2/3)μ (∇·u) δ_ij + ζ (∇·u) δ_ij`.
//!
//! The returned tensor is the *viscous* stress `τ`, packaged as a dedicated
//! [`ViscousStress<R>`] newtype to distinguish it from the full Cauchy
//! stress `σ = −p I + τ` at the type level. Downstream dissipation /
//! entropy-production kernels demand `ViscousStress`, so the `Φ = τ:∇u ≥ 0`
//! Clausius–Duhem guarantee is preserved by type-checking, not by docstring
//! discipline.

use crate::PhysicsError;
use crate::kernels::fluids::quantities::{StrainRateTensor, Viscosity, ViscousStress};
use deep_causality_num::{FromPrimitive, RealField};

/// Newtonian viscous stress with Stokes hypothesis (bulk viscosity ζ = 0):
/// `τ = 2μ S − (2/3) μ (∇·u) I`.
///
/// Returns a [`ViscousStress<R>`]. Symmetry is guaranteed by the algebra
/// (`S` is symmetric and the bulk term adds a scalar multiple of the
/// identity), but the checked constructor is used so that a non-finite
/// `div_u` is surfaced as `PhysicsError` rather than admitted silently.
pub fn newtonian_viscous_stress_kernel<R>(
    mu: &Viscosity<R>,
    strain_rate: &StrainRateTensor<R>,
    div_u: R,
) -> Result<ViscousStress<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let two_thirds = R::from_f64(2.0 / 3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0/3.0) failed".into()))?;
    let m = mu.value();
    let s = strain_rate.value();
    let bulk_term = two_thirds * m * div_u; // scalar multiplying the identity

    let mut tau = [[R::zero(); 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            tau[i][j] = two * m * s[i][j];
        }
    }
    tau[0][0] -= bulk_term;
    tau[1][1] -= bulk_term;
    tau[2][2] -= bulk_term;

    // Use the checked constructor: `div_u` is a raw R input, so a NaN/Inf
    // value would otherwise propagate into the diagonal silently. Symmetry
    // is guaranteed by the algebra (2μS symmetric + scalar·I).
    ViscousStress::new(tau)
}

/// Newtonian viscous stress with bulk viscosity:
/// `τ = 2μ S − (2/3) μ (∇·u) I + ζ (∇·u) I`.
///
/// Reduces to [`newtonian_viscous_stress_kernel`] when `ζ = 0`.
pub fn newtonian_viscous_stress_with_bulk_kernel<R>(
    mu: &Viscosity<R>,
    zeta: &Viscosity<R>,
    strain_rate: &StrainRateTensor<R>,
    div_u: R,
) -> Result<ViscousStress<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let two_thirds = R::from_f64(2.0 / 3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0/3.0) failed".into()))?;
    let m = mu.value();
    let z = zeta.value();
    let s = strain_rate.value();
    let bulk_term = (-two_thirds * m + z) * div_u;

    let mut tau = [[R::zero(); 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            tau[i][j] = two * m * s[i][j];
        }
    }
    tau[0][0] += bulk_term;
    tau[1][1] += bulk_term;
    tau[2][2] += bulk_term;

    // Use the checked constructor: `div_u` is a raw R input, so a NaN/Inf
    // value would otherwise propagate into the diagonal silently. Symmetry
    // is guaranteed by the algebra (2μS symmetric + scalar·I).
    ViscousStress::new(tau)
}

/// Power-law (Ostwald–de Waele) apparent viscosity: `μ_eff = K · γ̇^(n−1)`.
///
/// `K` is the consistency index (Pa·sⁿ); `n` is the flow-behaviour index.
/// At `n = 1` this reduces to Newtonian behaviour `μ_eff = K`.
///
/// Errors when `shear_rate < 0` (shear rate is a magnitude). When
/// `shear_rate = 0` and `n < 1`, the apparent viscosity diverges; the
/// non-finite result is caught by [`Viscosity::new`] and surfaces as a
/// `PhysicalInvariantBroken` error.
pub fn power_law_apparent_viscosity_kernel<R>(
    consistency: R,
    flow_index: R,
    shear_rate: R,
) -> Result<Viscosity<R>, PhysicsError>
where
    R: RealField,
{
    if shear_rate < R::zero() {
        return Err(PhysicsError::PhysicalInvariantBroken(
            "power_law_apparent_viscosity_kernel: shear_rate must be non-negative".into(),
        ));
    }
    let exponent = flow_index - R::one();
    let mu_eff = consistency * shear_rate.powf(exponent);
    Viscosity::new(mu_eff)
}
