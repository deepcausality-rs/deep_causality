/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coherent-structure detector kernels for fluid mechanics.
//!
//! Each kernel takes a [`VelocityGradient<R>`] (Jacobian convention
//! `[i][j] = ∂u_i/∂x_j`) and returns a scalar criterion used to identify
//! vortex cores in 3-D flow:
//!
//! - **Q-criterion** (Hunt, Wray & Moin 1988): `Q = 0.5 · (‖Ω‖² − ‖S‖²)`.
//! - **λ₂-criterion** (Jeong & Hussain 1995): the second-largest eigenvalue
//!   of the symmetric tensor `S² + Ω²`.
//! - **Δ-criterion** (Chong, Perry & Cantwell 1990): the discriminant
//!   `Δ = (Q/3)³ + (R/2)²` of the velocity-gradient characteristic polynomial
//!   in the trace-free (incompressible) limit, with `(Q, R)` from
//!   [`velocity_gradient_invariants_kernel`].
//! - **Swirling strength λ_ci** (Zhou et al. 1999): the imaginary part of the
//!   complex eigenvalue pair of `∇u`, or zero when all eigenvalues are real.
//!
//! Note: the "Q" of the Q-criterion is `0.5·(‖Ω‖² − ‖S‖²) = −0.5·tr(∇u · ∇u)`,
//! which equals the Chong–Perry–Cantwell `Q` invariant **only** for
//! incompressible flow (`tr(∇u) = 0`). For compressible flow the two differ
//! by `0.5·(tr(∇u))²`. The Δ-criterion below uses the CPC convention.

use crate::PhysicsError;
use crate::VelocityGradient;
use crate::kernels::fluids::kinematics::velocity_gradient_invariants_kernel;
use deep_causality_num::{FromPrimitive, RealField};

/// Q-criterion: `Q = 0.5 · (‖Ω‖² − ‖S‖²) = −0.5 · tr(∇u · ∇u)`.
///
/// Positive `Q` indicates regions where rotation dominates strain (vortex
/// cores). Reduces algebraically to `-0.5·tr(G²)` for any velocity gradient,
/// independent of compressibility.
pub fn q_criterion_kernel<R>(grad_u: &VelocityGradient<R>) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let g = grad_u.value();
    // tr(G²) = Σ_{i,j} G_ij · G_ji
    let tr_g_squared = g[0][0] * g[0][0]
        + g[0][1] * g[1][0]
        + g[0][2] * g[2][0]
        + g[1][0] * g[0][1]
        + g[1][1] * g[1][1]
        + g[1][2] * g[2][1]
        + g[2][0] * g[0][2]
        + g[2][1] * g[1][2]
        + g[2][2] * g[2][2];
    Ok(-half * tr_g_squared)
}

/// Δ-criterion (Chong, Perry & Cantwell 1990; generalized form): the
/// discriminant of the *depressed* velocity-gradient characteristic polynomial
/// `Δ = (p̃/3)³ + (q̃/2)²` where
/// `p̃ = Q − P²/3`, `q̃ = 2P³/27 − PQ/3 + R`,
/// and `(P, Q, R)` are the velocity-gradient invariants in the CPC convention.
///
/// `Δ > 0`: complex eigenvalues exist ⇒ rotational (focus/swirl) topology.
/// `Δ < 0`: three distinct real eigenvalues ⇒ non-rotational topology.
/// `Δ = 0`: repeated real eigenvalues (discriminant boundary).
///
/// In the incompressible limit `P = 0`, `(p̃, q̃) = (Q, R)` and this reduces to
/// the classical `Δ = (Q/3)³ + (R/2)²`. The general form here uses the
/// trace-shifted invariants so the criterion is correct for compressible
/// flow as well, matching the convention in Chakraborty, Balachandar & Adrian
/// (2005).
pub fn delta_criterion_kernel<R>(grad_u: &VelocityGradient<R>) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let third = R::from_f64(1.0 / 3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(1/3) failed".into()))?;
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let twenty_seven = R::from_f64(27.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(27.0) failed".into()))?;

    let (p, q, r) = velocity_gradient_invariants_kernel(grad_u)?;
    // Depressed cubic coefficients (substitution λ = y − P/3).
    let p_d = q - p * p * third;
    let q_d = two * p * p * p / twenty_seven - p * q * third + r;
    let p3 = p_d * third;
    let q2 = q_d * half;
    Ok(p3 * p3 * p3 + q2 * q2)
}

/// λ₂-criterion (Jeong & Hussain 1995): the **second-largest** (middle)
/// eigenvalue of the symmetric tensor `M = S² + Ω²` where `S` and `Ω` are
/// the symmetric and antisymmetric parts of `∇u`.
///
/// `λ₂ < 0` identifies a vortex core. Implemented via the closed-form Smith
/// (1961) algorithm for 3×3 symmetric eigenvalues.
pub fn lambda2_kernel<R>(grad_u: &VelocityGradient<R>) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let g = grad_u.value();

    // S = 0.5 (G + Gᵀ), Ω = 0.5 (G − Gᵀ).
    let mut s = [[R::zero(); 3]; 3];
    let mut o = [[R::zero(); 3]; 3];
    for (i, (s_row, o_row)) in s.iter_mut().zip(o.iter_mut()).enumerate() {
        for (j, (s_ij, o_ij)) in s_row.iter_mut().zip(o_row.iter_mut()).enumerate() {
            *s_ij = half * (g[i][j] + g[j][i]);
            *o_ij = half * (g[i][j] - g[j][i]);
        }
    }
    // M = S² + Ω² (symmetric).
    let m = sym_3x3_add(&mat3_mul(&s, &s), &mat3_mul(&o, &o));
    let eigs = symmetric_3x3_eigenvalues(&m)?;
    // Middle eigenvalue (sorted descending → index 1).
    Ok(eigs[1])
}

/// Swirling strength `λ_ci` (Zhou et al. 1999): the imaginary part of the
/// complex eigenvalue pair of `∇u`, or zero when all eigenvalues are real.
///
/// Computed from the discriminant of the velocity-gradient characteristic
/// polynomial via Cardano's formula on the depressed cubic. Returns
/// the non-negative magnitude `|Im(λ)|`.
pub fn swirling_strength_kernel<R>(grad_u: &VelocityGradient<R>) -> Result<R, PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let third = R::from_f64(1.0 / 3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(1/3) failed".into()))?;
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let three = R::from_f64(3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(3.0) failed".into()))?;
    let twenty_seven = R::from_f64(27.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(27.0) failed".into()))?;

    let (p, q, r) = velocity_gradient_invariants_kernel(grad_u)?;

    // Characteristic polynomial of A: λ³ − tr(A)·λ² + I2·λ − det(A) = 0.
    // In CPC notation: λ³ + P·λ² + Q·λ + R = 0 since P = -tr(A), Q = I2, R = -det(A).
    // Substitute λ = y − P/3 to depress: y³ + p_d·y + q_d = 0.
    let p_d = q - p * p * third;
    let q_d = two * p * p * p / twenty_seven - p * q * third + r;

    // Discriminant of the depressed cubic: (q_d/2)² + (p_d/3)³.
    // > 0  → 1 real + 2 complex conjugates (vortex regime)
    // ≤ 0  → 3 real roots  →  swirling strength = 0
    let disc = (q_d * half) * (q_d * half) + (p_d * third) * (p_d * third) * (p_d * third);
    if disc <= R::zero() {
        return Ok(R::zero());
    }

    // Real cube roots u₁, u₂ of (−q_d/2 ± √disc).
    let sqrt_disc = disc.sqrt();
    let u1 = signed_cbrt(-q_d * half + sqrt_disc, third);
    let u2 = signed_cbrt(-q_d * half - sqrt_disc, third);

    // Complex part of the eigenvalue pair: ±(√3 / 2) · (u₁ − u₂).
    let sqrt_3_over_2 = three.sqrt() * half;
    let diff = u1 - u2;
    let abs_diff = if diff < R::zero() { -diff } else { diff };
    Ok(sqrt_3_over_2 * abs_diff)
}

// =============================================================================
// Private helpers
// =============================================================================

fn mat3_mul<R: RealField>(a: &[[R; 3]; 3], b: &[[R; 3]; 3]) -> [[R; 3]; 3] {
    let mut out = [[R::zero(); 3]; 3];
    for (i, out_row) in out.iter_mut().enumerate() {
        for (j, out_ij) in out_row.iter_mut().enumerate() {
            *out_ij = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

fn sym_3x3_add<R: RealField>(a: &[[R; 3]; 3], b: &[[R; 3]; 3]) -> [[R; 3]; 3] {
    let mut out = [[R::zero(); 3]; 3];
    for (i, out_row) in out.iter_mut().enumerate() {
        for (j, out_ij) in out_row.iter_mut().enumerate() {
            *out_ij = a[i][j] + b[i][j];
        }
    }
    out
}

/// Real cube root that preserves sign: `signed_cbrt(x) = sign(x) · |x|^(1/3)`.
fn signed_cbrt<R: RealField>(x: R, one_third: R) -> R {
    if x >= R::zero() {
        x.powf(one_third)
    } else {
        -((-x).powf(one_third))
    }
}

/// Eigenvalues of a real symmetric 3×3 matrix, sorted descending.
///
/// Closed-form Smith (1961) algorithm: handles the diagonal case directly,
/// otherwise reduces to the cubic on the trace-shifted, normalised matrix.
fn symmetric_3x3_eigenvalues<R>(m: &[[R; 3]; 3]) -> Result<[R; 3], PhysicsError>
where
    R: RealField + FromPrimitive,
{
    let third = R::from_f64(1.0 / 3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(1/3) failed".into()))?;
    let half = R::from_f64(0.5)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(0.5) failed".into()))?;
    let two = R::from_f64(2.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2.0) failed".into()))?;
    let six = R::from_f64(6.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(6.0) failed".into()))?;
    let three = R::from_f64(3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(3.0) failed".into()))?;
    let two_pi_over_3 = R::from_f64(2.0 * core::f64::consts::PI / 3.0)
        .ok_or_else(|| PhysicsError::NumericalInstability("R::from_f64(2π/3) failed".into()))?;

    let p1 = m[0][1] * m[0][1] + m[0][2] * m[0][2] + m[1][2] * m[1][2];
    if p1 == R::zero() {
        // Diagonal matrix; eigenvalues are the diagonal entries.
        let mut e = [m[0][0], m[1][1], m[2][2]];
        sort_desc_3(&mut e);
        return Ok(e);
    }

    let q = (m[0][0] + m[1][1] + m[2][2]) * third; // trace / 3
    let d00 = m[0][0] - q;
    let d11 = m[1][1] - q;
    let d22 = m[2][2] - q;
    let p2 = d00 * d00 + d11 * d11 + d22 * d22 + two * p1;
    let p = (p2 / six).sqrt();
    // B = (1/p) (M − q·I)
    let inv_p = R::one() / p;
    let b00 = d00 * inv_p;
    let b11 = d11 * inv_p;
    let b22 = d22 * inv_p;
    let b01 = m[0][1] * inv_p;
    let b02 = m[0][2] * inv_p;
    let b12 = m[1][2] * inv_p;
    // det(B)
    let det_b = b00 * (b11 * b22 - b12 * b12) - b01 * (b01 * b22 - b12 * b02)
        + b02 * (b01 * b12 - b11 * b02);
    let mut r_val = det_b * half;
    // Clamp r into [-1, 1] to absorb floating-point overshoot before acos.
    if r_val < -R::one() {
        r_val = -R::one();
    }
    if r_val > R::one() {
        r_val = R::one();
    }
    let phi = r_val.acos() * third;

    let eig1 = q + two * p * phi.cos();
    let eig3 = q + two * p * (phi + two_pi_over_3).cos();
    let eig2 = three * q - eig1 - eig3;

    let mut e = [eig1, eig2, eig3];
    sort_desc_3(&mut e);
    Ok(e)
}

fn sort_desc_3<R: RealField>(a: &mut [R; 3]) {
    if a[0] < a[1] {
        a.swap(0, 1);
    }
    if a[1] < a[2] {
        a.swap(1, 2);
    }
    if a[0] < a[1] {
        a.swap(0, 1);
    }
}
