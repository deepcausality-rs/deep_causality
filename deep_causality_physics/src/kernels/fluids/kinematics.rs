/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Pointwise kinematic kernels for fluid mechanics.
//!
//! # Velocity gradient convention
//!
//! All kernels in this module consume a [`VelocityGradient<R>`] whose
//! underlying matrix follows the Jacobian convention `[i][j] = ∂u_i / ∂x_j`.
//! The newtype pins this convention at the call site; see `quantities.rs`
//! for the type definition.

use crate::PhysicsError;
use crate::{RotationRateTensor, StrainRateTensor, Velocity3, VelocityGradient, VorticityVector};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

/// Strain-rate tensor `S = 0.5 · (∇u + ∇uᵀ)`.
///
/// Symmetric part of the velocity gradient. Captures the rate of deformation
/// of fluid elements (stretching, shearing) independent of rigid-body rotation.
/// Vanishes for rigid-body motion.
///
/// Returned via [`StrainRateTensor::new_unchecked`]: the algebra of
/// `0.5·(G + Gᵀ)` guarantees `S_ij == S_ji` exactly in IEEE 754, so the
/// construction-time symmetry check is redundant here.
pub fn strain_rate_tensor_kernel<R>(
    grad_u: &VelocityGradient<R>,
) -> Result<StrainRateTensor<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let g = grad_u.value();
    let mut s = [[R::zero(); 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            s[i][j] = half * (g[i][j] + g[j][i]);
        }
    }
    Ok(StrainRateTensor::new_unchecked(s))
}

/// Rate-of-rotation (spin) tensor `Ω = 0.5 · (∇u − ∇uᵀ)`.
///
/// Antisymmetric part of the velocity gradient. Captures the rate of rigid-body
/// rotation of fluid elements. Vanishes for irrotational flow.
///
/// Returned via [`RotationRateTensor::new_unchecked`]: the algebra of
/// `0.5·(G − Gᵀ)` guarantees antisymmetry exactly in IEEE 754.
pub fn rotation_rate_tensor_kernel<R>(
    grad_u: &VelocityGradient<R>,
) -> Result<RotationRateTensor<R>, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let g = grad_u.value();
    let mut omega = [[R::zero(); 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            omega[i][j] = half * (g[i][j] - g[j][i]);
        }
    }
    Ok(RotationRateTensor::new_unchecked(omega))
}

/// Vorticity vector `ω = ∇ × u`, computed from the velocity gradient.
///
/// Component formulas (Jacobian convention):
/// - `ω_x = ∂u_z/∂y − ∂u_y/∂z = grad_u[2][1] − grad_u[1][2]`
/// - `ω_y = ∂u_x/∂z − ∂u_z/∂x = grad_u[0][2] − grad_u[2][0]`
/// - `ω_z = ∂u_y/∂x − ∂u_x/∂y = grad_u[1][0] − grad_u[0][1]`
///
/// Equivalent to twice the axial vector of the rate-of-rotation tensor:
/// `ω_i = ε_{ijk} Ω_{jk}`. Vanishes for irrotational flow.
pub fn vorticity_from_gradient_kernel<R>(grad_u: &VelocityGradient<R>) -> VorticityVector<R>
where
    R: RealField,
{
    let g = grad_u.value();
    VorticityVector::new_unchecked([g[2][1] - g[1][2], g[0][2] - g[2][0], g[1][0] - g[0][1]])
}

/// Invariants `(P, Q, R)` of the velocity gradient tensor `A = ∇u`,
/// following the Chong–Perry–Cantwell (1990) convention used in vortex
/// classification:
///
/// - `P = −tr(A) = −div(u)`
/// - `Q = 0.5 · (P² − tr(A²))`
/// - `R = −det(A)`
///
/// For incompressible flow `P = 0`, and `(Q, R)` parametrise the standard
/// vortex-classification chart.
pub fn velocity_gradient_invariants_kernel<R>(
    grad_u: &VelocityGradient<R>,
) -> Result<(R, R, R), PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let g = grad_u.value();

    // P = -tr(A)
    let trace_a = g[0][0] + g[1][1] + g[2][2];
    let p = -trace_a;

    // tr(A^2) = sum_{i,j} A_{ij} * A_{ji}, unrolled for the 3x3 case.
    let trace_a_squared = g[0][0] * g[0][0]
        + g[0][1] * g[1][0]
        + g[0][2] * g[2][0]
        + g[1][0] * g[0][1]
        + g[1][1] * g[1][1]
        + g[1][2] * g[2][1]
        + g[2][0] * g[0][2]
        + g[2][1] * g[1][2]
        + g[2][2] * g[2][2];
    let q = half * (p * p - trace_a_squared);

    // R = -det(A), 3x3 determinant by cofactor expansion along row 0
    let det_a = g[0][0] * (g[1][1] * g[2][2] - g[1][2] * g[2][1])
        - g[0][1] * (g[1][0] * g[2][2] - g[1][2] * g[2][0])
        + g[0][2] * (g[1][0] * g[2][1] - g[1][1] * g[2][0]);
    let r = -det_a;

    Ok((p, q, r))
}

/// Helicity density `h = u · ω`.
///
/// Signed scalar measuring local alignment of velocity and vorticity vectors.
/// Flips sign under spatial reflection (helicity is a pseudoscalar).
pub fn helicity_density_kernel<R>(u: &Velocity3<R>, omega: &VorticityVector<R>) -> R
where
    R: RealField,
{
    let u_raw = u.value();
    let w_raw = omega.value();
    u_raw[0] * w_raw[0] + u_raw[1] * w_raw[1] + u_raw[2] * w_raw[2]
}

/// Enstrophy density `ξ = 0.5 · ‖ω‖²`.
///
/// Non-negative scalar measuring local rotational kinetic energy density.
pub fn enstrophy_density_kernel<R>(omega: &VorticityVector<R>) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let w = omega.value();
    Ok(half * (w[0] * w[0] + w[1] * w[1] + w[2] * w[2]))
}
