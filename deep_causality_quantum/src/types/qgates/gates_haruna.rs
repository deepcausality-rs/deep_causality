/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::QuantumError;
use core::f64::consts::PI;
use deep_causality_algebra::{DivisionAlgebra, RealField};
/// Haruna's Gauge Field Formalism for Logical Quantum Gates.
///
/// Based on "Note on Logical Gates by Gauge Field Formalism of Quantum Error Correction"
/// by Junichi Haruna (2025).
/// See: https://arxiv.org/abs/2511.15224
use deep_causality_multivector::CausalMultiVector;
use deep_causality_num::{FromPrimitive, One};
use deep_causality_num_complex::Complex;

/// Whether any component of `mv` has a non-finite real or imaginary part. Shared
/// by the finiteness guards in `exp` and `logical_hadamard` (mirroring the checks
/// in `apply_gate_kernel` / `commutator_kernel`) so a new post-product path cannot
/// silently forget the guard.
fn has_non_finite<R: RealField>(mv: &CausalMultiVector<Complex<R>>) -> bool {
    mv.data()
        .iter()
        .any(|c| !c.re.is_finite() || !c.im.is_finite())
}

/// Helper function to compute the exponential of a multivector: $e^A = \sum A^n/n!$
/// Uses Taylor series expansion.
///
/// # Errors
/// Returns [`QuantumError::NonFiniteValue`] when the exponent's norm is
/// non-finite or exceeds the overflow bound, or when the Taylor series overflows
/// to a non-finite value during (or after) accumulation; returns
/// [`QuantumError::CalculationError`] when the 64-term series does not converge to
/// tolerance within the budget (the truncated sum would be silently inaccurate).
/// An overflowing or non-convergent exponent has no reliable finite `exp`, so the
/// failure is surfaced rather than masked as the identity operator — a real
/// identity gate (`exp(0)`) is indistinguishable from such a fallback, which is
/// exactly the ambiguity callers must not face.
fn exp<R>(mv: &CausalMultiVector<Complex<R>>) -> Result<CausalMultiVector<Complex<R>>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let small = R::from_f64(1e-15).expect("R::from_f64(1e-15)");
    let tol = R::from_f64(1e-12).expect("R::from_f64(1e-12)");
    let max_norm = R::from_f64(1e6).expect("R::from_f64(1e6)");

    // Fast path: exp(0) = I. This is a genuine identity result, not a fallback.
    if mv
        .data()
        .iter()
        .all(|c| c.re.abs() < small && c.im.abs() < small)
    {
        return Ok(CausalMultiVector::scalar(Complex::one(), mv.metric()));
    }

    let metric = mv.metric();
    let one_complex = Complex::<R>::one();
    let mut term = CausalMultiVector::scalar(one_complex, metric);
    let mut sum = term.clone();

    let max_iters = 64;

    // A huge exponent has no finite exp; reject it instead of masking as identity.
    let mv_norm_sq: R = mv
        .data()
        .iter()
        .map(|c| c.norm_sqr())
        .fold(R::zero(), |acc, x| acc + x);
    let mv_norm = mv_norm_sq.sqrt();
    if !mv_norm.is_finite() || mv_norm > max_norm {
        return Err(QuantumError::NonFiniteValue(
            "Haruna gate exponent norm is non-finite or exceeds the overflow bound; \
             the gate has no finite value"
                .into(),
        ));
    }

    for n in 1..=max_iters {
        let n_r = R::from_f64(n as f64).expect("R::from_f64(n)");
        let n_inv = Complex::new(R::one() / n_r, R::zero());
        term = &term * mv;
        term *= n_inv;

        if has_non_finite(&term) {
            return Err(QuantumError::NonFiniteValue(
                "Haruna gate exp Taylor term overflowed to a non-finite value".into(),
            ));
        }

        let prev = sum.clone();
        sum += &term;

        let diff = &sum - &prev;
        let delta_sq: R = diff
            .data()
            .iter()
            .map(|c| c.norm_sqr())
            .fold(R::zero(), |acc, x| acc + x);
        let delta = delta_sq.sqrt();

        if !delta.is_finite() {
            return Err(QuantumError::NonFiniteValue(
                "Haruna gate exp Taylor series overflowed to a non-finite value".into(),
            ));
        }
        if delta < tol {
            return Ok(sum);
        }
    }

    // Reaching here means the 64-term budget was exhausted WITHOUT `delta < tol`
    // (convergence returns `Ok` inside the loop). The truncated partial sum is a
    // non-converged Taylor series — for an exponent norm above ~16 it can be many
    // orders of magnitude off — so surface it as a failure rather than a
    // silently-wrong gate. A `sum` can also overflow to a non-finite value purely
    // by accumulating finite terms; report that case distinctly.
    if has_non_finite(&sum) {
        return Err(QuantumError::NonFiniteValue(
            "Haruna gate exp result is non-finite".into(),
        ));
    }
    Err(QuantumError::CalculationError(
        "Haruna gate exp Taylor series did not converge within the 64-term budget \
         to tolerance 1e-12; the exponent norm is too large (the field is outside \
         the reliably-computable domain)"
            .into(),
    ))
}

/// Calculates the Logical Z gate: $Z(\gamma) = \exp(i \pi a(\gamma))$.
///
/// # Errors
/// Propagates `exp`'s errors: [`QuantumError::NonFiniteValue`] on overflow, or
/// [`QuantumError::CalculationError`] if the Taylor series does not converge.
pub fn logical_z<R>(
    a_gamma: &CausalMultiVector<Complex<R>>,
) -> Result<CausalMultiVector<Complex<R>>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let pi = R::from_f64(PI).expect("R::from_f64(PI)");
    let i_pi = Complex::new(R::zero(), pi);
    let exponent = a_gamma.clone() * i_pi;
    exp(&exponent)
}

/// Calculates the Logical X gate: $X(\tilde{\gamma}) = \exp(i \pi b(\tilde{\gamma}))$.
///
/// # Errors
/// Propagates `exp`'s errors: [`QuantumError::NonFiniteValue`] on overflow, or
/// [`QuantumError::CalculationError`] if the Taylor series does not converge.
pub fn logical_x<R>(
    b_gamma_tilde: &CausalMultiVector<Complex<R>>,
) -> Result<CausalMultiVector<Complex<R>>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let pi = R::from_f64(PI).expect("R::from_f64(PI)");
    let i_pi = Complex::new(R::zero(), pi);
    let exponent = b_gamma_tilde.clone() * i_pi;
    exp(&exponent)
}

/// Calculates the Logical S gate: $S(\gamma) = \exp(i \frac{\pi}{2} a(\gamma)^2)$.
///
/// # Errors
/// Propagates `exp`'s errors: [`QuantumError::NonFiniteValue`] on overflow, or
/// [`QuantumError::CalculationError`] if the Taylor series does not converge.
pub fn logical_s<R>(
    a_gamma: &CausalMultiVector<Complex<R>>,
) -> Result<CausalMultiVector<Complex<R>>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let pi_2 = R::from_f64(PI / 2.0).expect("R::from_f64(PI/2)");
    let i_pi_2 = Complex::new(R::zero(), pi_2);
    let a_sq = a_gamma.clone() * a_gamma.clone();
    let exponent = a_sq * i_pi_2;
    exp(&exponent)
}

/// Calculates the Logical Hadamard gate.
///
/// # Errors
/// Propagates `exp`'s errors (via `logical_s` and the mid-factor), and returns
/// [`QuantumError::NonFiniteValue`] if the post-exponential products overflow to a
/// non-finite value.
pub fn logical_hadamard<R>(
    a_gamma: &CausalMultiVector<Complex<R>>,
    b_gamma_tilde: &CausalMultiVector<Complex<R>>,
) -> Result<CausalMultiVector<Complex<R>>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    // phase_scalar = exp(-i pi/4) = cos(-pi/4) + i sin(-pi/4)
    let neg_pi_4 = R::from_f64(-PI / 4.0).expect("R::from_f64(-PI/4)");
    let phase_scalar = Complex::new(neg_pi_4.cos(), neg_pi_4.sin());

    let s_a = logical_s(a_gamma)?;

    let pi_2 = R::from_f64(PI / 2.0).expect("R::from_f64(PI/2)");
    let i_pi_2 = Complex::new(R::zero(), pi_2);
    let b_sq = b_gamma_tilde.clone() * b_gamma_tilde.clone();
    let mid = exp(&(b_sq * i_pi_2))?;

    let step1 = &s_a * &mid;
    let step2 = &step1 * &s_a;
    // The post-exp geometric products can overflow to a non-finite value even when
    // both exp factors are finite; backstop them (consistent with exp,
    // apply_gate_kernel, and commutator_kernel).
    let result = step2 * phase_scalar;
    if has_non_finite(&result) {
        return Err(QuantumError::NonFiniteValue(
            "Haruna Hadamard gate produced a non-finite value".into(),
        ));
    }
    Ok(result)
}

/// Calculates the Logical CZ gate: $CZ(\gamma_1, \gamma_2) = \exp(i \pi a(\gamma_1) a(\gamma_2))$.
///
/// # Errors
/// Propagates `exp`'s errors: [`QuantumError::NonFiniteValue`] on overflow, or
/// [`QuantumError::CalculationError`] if the Taylor series does not converge.
pub fn logical_cz<R>(
    a_gamma1: &CausalMultiVector<Complex<R>>,
    a_gamma2: &CausalMultiVector<Complex<R>>,
) -> Result<CausalMultiVector<Complex<R>>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let pi = R::from_f64(PI).expect("R::from_f64(PI)");
    let i_pi = Complex::new(R::zero(), pi);
    let interaction = a_gamma1.clone() * a_gamma2.clone();
    let exponent = interaction * i_pi;
    exp(&exponent)
}

/// Calculates the Logical T gate.
///
/// # Errors
/// Propagates `exp`'s errors: [`QuantumError::NonFiniteValue`] on overflow, or
/// [`QuantumError::CalculationError`] if the Taylor series does not converge.
pub fn logical_t<R>(
    a_gamma: &CausalMultiVector<Complex<R>>,
) -> Result<CausalMultiVector<Complex<R>>, QuantumError>
where
    R: RealField + FromPrimitive,
{
    let a = a_gamma.clone();
    let a2 = &a * &a;
    let a3 = &a2 * &a;

    let half = R::from_f64(0.5).expect("R::from_f64(0.5)");
    let neg_three_quarters = R::from_f64(-0.75).expect("R::from_f64(-0.75)");

    let c1 = Complex::new(half, R::zero());
    let c2 = Complex::new(neg_three_quarters, R::zero());
    let c3 = Complex::new(half, R::zero());

    let term1 = a3 * c1;
    let term2 = a2 * c2;
    let term3 = a * c3;

    let sum = term1 + term2 + term3;
    let pi = R::from_f64(PI).expect("R::from_f64(PI)");
    let i_pi = Complex::new(R::zero(), pi);

    exp(&(sum * i_pi))
}
