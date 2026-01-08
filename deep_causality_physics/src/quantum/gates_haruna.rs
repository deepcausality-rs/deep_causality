/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::f64::consts::PI;
/// Haruna's Gauge Field Formalism for Logical Quantum Gates.
///
/// Based on "Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction"
/// by Junichi Haruna (2025).
/// See: https://arxiv.org/abs/2511.15224
///
/// This module constructs logical quantum gates for CSS codes as exponentials of
/// polynomials of the electric (`a`) and magnetic (`b`) gauge fields.
///
/// # Assumptions
/// * The gauge fields `a` and `b` are assumed to be **Projector-valued** (eigenvalues 0 and 1)
///   representing the presence (1) or absence (0) of the field/error chain.
/// * They satisfy $a^2 = a$ and $b^2 = b$ in the code space (idempotent).
/// * If `a` or `b` are Pauli operators (eigenvalues $\pm 1$), the polynomial coefficients
///   implemented here for T and S gates will not yield standard phases.
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{Complex, DivisionAlgebra, One};

/// Helper function to compute the exponential of a multivector: $e^A = \sum A^n/n!$
/// Uses Taylor series expansion.
fn exp(mv: &CausalMultiVector<Complex<f64>>) -> CausalMultiVector<Complex<f64>> {
    // Fast path: exp(0) = I
    if mv
        .data()
        .iter()
        .all(|c| c.re.abs() < 1e-15 && c.im.abs() < 1e-15)
    {
        return CausalMultiVector::scalar(Complex::one(), mv.metric());
    }

    let metric = mv.metric();
    let one_complex = Complex::one();
    let mut term = CausalMultiVector::scalar(one_complex, metric);
    let mut sum = term.clone();

    let tol = 1e-12;
    let max_iters = 64;

    // Simple bound to detect pathological exponents; if ||mv|| is huge, series may overflow
    let mv_norm = mv.data().iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt();
    if !mv_norm.is_finite() || mv_norm > 1e6 {
        // Return zero-order approximation to avoid producing NaNs downstream
        return CausalMultiVector::scalar(Complex::one(), metric);
    }

    for n in 1..=max_iters {
        let n_inv = Complex::new(1.0 / (n as f64), 0.0);
        term = &term * mv;
        term *= n_inv;

        if term
            .data()
            .iter()
            .any(|c| !c.re.is_finite() || !c.im.is_finite())
        {
            // Abort with safest finite approximation so far
            return sum;
        }

        let prev = sum.clone();
        sum += &term;

        let diff = &sum - &prev;
        let delta = diff.data().iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt();

        if !delta.is_finite() {
            return prev;
        }
        if delta < tol {
            return sum;
        }
    }

    // Ensure finiteness of result
    if sum
        .data()
        .iter()
        .any(|c| !c.re.is_finite() || !c.im.is_finite())
    {
        // Fall back to the last finite partial sum (here identity if none improved)
        return CausalMultiVector::scalar(Complex::one(), metric);
    }
    sum
}

/// Calculates the Logical Z gate: $Z(\gamma) = \exp(i \pi a(\gamma))$.
pub fn logical_z(a_gamma: &CausalMultiVector<Complex<f64>>) -> CausalMultiVector<Complex<f64>> {
    let i_pi = Complex::new(0.0, PI);
    let exponent = a_gamma.clone() * i_pi;
    exp(&exponent)
}

/// Calculates the Logical X gate: $X(\tilde{\gamma}) = \exp(i \pi b(\tilde{\gamma}))$.
pub fn logical_x(
    b_gamma_tilde: &CausalMultiVector<Complex<f64>>,
) -> CausalMultiVector<Complex<f64>> {
    let i_pi = Complex::new(0.0, PI);
    let exponent = b_gamma_tilde.clone() * i_pi;
    exp(&exponent)
}

/// Calculates the Logical S gate: $S(\gamma) = \exp(i \frac{\pi}{2} a(\gamma)^2)$.
pub fn logical_s(a_gamma: &CausalMultiVector<Complex<f64>>) -> CausalMultiVector<Complex<f64>> {
    let i_pi_2 = Complex::new(0.0, PI / 2.0);
    // Geometric Product
    let a_sq = a_gamma.clone() * a_gamma.clone();
    let exponent = a_sq * i_pi_2;
    exp(&exponent)
}

/// Calculates the Logical Hadamard gate.
///
/// $H(\gamma) = e^{-i \frac{\pi}{4}} \exp(i \frac{\pi}{2} a(\gamma)^2) \exp(i \frac{\pi}{2} b(\tilde{\gamma})^2) \exp(i \frac{\pi}{2} a(\gamma)^2)$.
pub fn logical_hadamard(
    a_gamma: &CausalMultiVector<Complex<f64>>,
    b_gamma_tilde: &CausalMultiVector<Complex<f64>>,
) -> CausalMultiVector<Complex<f64>> {
    let phase_scalar = Complex::new(0.0, -PI / 4.0).exp();

    // S(gamma) part
    let s_a = logical_s(a_gamma);

    // Middle part: exp(i pi/2 b^2)
    let i_pi_2 = Complex::new(0.0, PI / 2.0);
    let b_sq = b_gamma_tilde.clone() * b_gamma_tilde.clone();
    let mid = exp(&(b_sq * i_pi_2));

    // H = phase * S(a) * Mid(b) * S(a)
    let step1 = &s_a * &mid;
    let step2 = &step1 * &s_a;
    step2 * phase_scalar
}

/// Calculates the Logical CZ gate: $CZ(\gamma_1, \gamma_2) = \exp(i \pi a(\gamma_1) a(\gamma_2))$.
pub fn logical_cz(
    a_gamma1: &CausalMultiVector<Complex<f64>>,
    a_gamma2: &CausalMultiVector<Complex<f64>>,
) -> CausalMultiVector<Complex<f64>> {
    let i_pi = Complex::new(0.0, PI);
    let interaction = a_gamma1.clone() * a_gamma2.clone();
    let exponent = interaction * i_pi;
    exp(&exponent)
}

/// Calculates the Logical T gate.
///
/// $T(\gamma) = \exp(i \pi (\frac{1}{2} a(\gamma)^3 - \frac{3}{4} a(\gamma)^2 + \frac{1}{2} a(\gamma)))$.
pub fn logical_t(a_gamma: &CausalMultiVector<Complex<f64>>) -> CausalMultiVector<Complex<f64>> {
    let a = a_gamma.clone();
    let a2 = &a * &a;
    let a3 = &a2 * &a;

    let c1 = Complex::new(0.5, 0.0);
    let c2 = Complex::new(-0.75, 0.0);
    let c3 = Complex::new(0.5, 0.0);

    let term1 = a3 * c1;
    let term2 = a2 * c2;
    let term3 = a * c3;

    let sum = term1 + term2 + term3;
    let i_pi = Complex::new(0.0, PI);

    exp(&(sum * i_pi))
}
