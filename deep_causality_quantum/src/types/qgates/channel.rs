/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The Choi–Jamiołkowski isomorphism and the Kraus representation (L3 of the
//! operator build ladder, design B4).
//!
//! Conventions (fixed by the tests): the Choi operator of a channel
//! `E: L(H_in) → L(H_out)` is `J(E) = Σ_{ik} |i⟩⟨k| ⊗ E(|i⟩⟨k|)`, a
//! `(d_in·d_out)`-square matrix with the composite row index `(i·d_out + j)`.
//! For a Kraus family `{K_κ}`: `J[(i,j),(k,l)] = Σ_κ K_κ[j,i]·conj(K_κ[l,k])`.
//! `E` is completely positive iff `J ⪰ 0`, and trace-preserving iff
//! `Tr_out(J) = I_in`.
//!
//! Reference: R. Lorenz, "Quantum causal models: the merits of the spirit of
//! Reichenbach's principle" (2022); M.-D. Choi, "Completely positive linear
//! maps on complex matrices", Linear Algebra Appl. 10 (1975) 285–290.

use crate::QuantumError;
use crate::types::qgates::operator_linalg::{
    hermiticity_defect, identity_matrix, partial_trace, square_dim,
};
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, Tensor};

fn cmul<R: RealField>(a: Complex<R>, b: Complex<R>) -> Complex<R> {
    Complex::new(a.re * b.re - a.im * b.im, a.re * b.im + a.im * b.re)
}

fn conj<R: RealField>(a: Complex<R>) -> Complex<R> {
    Complex::new(a.re, -a.im)
}

/// Rejects a non-finite or negative validation tolerance. A NaN/∞ `tol` makes
/// every `defect > tol` and `eigenvalue < -tol` comparison vacuously false, which
/// would silently certify a non-Hermitian / non-PSD / non-TP Choi operator.
fn validate_tolerance<R: RealField>(tol: R) -> Result<(), QuantumError> {
    if !tol.is_finite() || tol < R::zero() {
        return Err(QuantumError::CalculationError(
            "validation tolerance must be finite and non-negative".into(),
        ));
    }
    Ok(())
}

/// Builds the Choi operator `J(E)` from a Kraus family `{K_κ}` of `d_out×d_in`
/// matrices: `J[(i,j),(k,l)] = Σ_κ K_κ[j,i]·conj(K_κ[l,k])`.
pub fn choi_from_kraus<R>(
    kraus: &[CausalTensor<Complex<R>>],
) -> Result<CausalTensor<Complex<R>>, QuantumError>
where
    R: RealField,
{
    if kraus.is_empty() {
        return Err(QuantumError::NonCptpChannel("empty Kraus family".into()));
    }
    let shape = kraus[0].shape();
    if shape.len() != 2 || shape[0] == 0 || shape[1] == 0 {
        return Err(QuantumError::DimensionMismatch(format!(
            "Kraus operators must be non-empty matrices, got shape {:?}",
            shape
        )));
    }
    let (d_out, d_in) = (shape[0], shape[1]);
    if kraus.iter().any(|k| k.shape() != shape) {
        return Err(QuantumError::DimensionMismatch(
            "Kraus operators disagree on shape".into(),
        ));
    }

    let d = d_in * d_out;
    let mut j = vec![Complex::new(R::zero(), R::zero()); d * d];
    for k_op in kraus {
        let ks = k_op.as_slice();
        for i in 0..d_in {
            for jj in 0..d_out {
                let a = ks[jj * d_in + i]; // K[j, i]
                for k in 0..d_in {
                    for l in 0..d_out {
                        let b = conj(ks[l * d_in + k]); // conj(K[l, k])
                        let row = i * d_out + jj;
                        let col = k * d_out + l;
                        let v = cmul(a, b);
                        let cur = j[row * d + col];
                        j[row * d + col] = Complex::new(cur.re + v.re, cur.im + v.im);
                    }
                }
            }
        }
    }
    Ok(CausalTensor::from_slice(&j, &[d, d]))
}

/// Recovers a Kraus family from a (PSD) Choi operator via the Hermitian
/// eigendecomposition: `K_κ = √λ_κ · unvec(v_κ)` for `λ_κ > tol`.
pub fn kraus_from_choi<R>(
    choi: &CausalTensor<Complex<R>>,
    d_in: usize,
    d_out: usize,
    tol: R,
) -> Result<Vec<CausalTensor<Complex<R>>>, QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    validate_tolerance(tol)?;
    let d = square_dim(choi)?;
    if d != d_in * d_out {
        return Err(QuantumError::DimensionMismatch(format!(
            "Choi dim {} != d_in·d_out = {}",
            d,
            d_in * d_out
        )));
    }
    // eigen_hermitian silently decomposes only the Hermitian part of its input,
    // so a non-Hermitian (or non-finite) Choi would be reconstructed from the
    // wrong operator. Reject both up front — PSD requires Hermiticity.
    if choi
        .as_slice()
        .iter()
        .any(|c| !c.re.is_finite() || !c.im.is_finite())
    {
        return Err(QuantumError::NonFiniteValue(
            "Choi operator contains a non-finite entry".into(),
        ));
    }
    let defect = hermiticity_defect(choi)?;
    if defect > tol {
        return Err(QuantumError::NonPositiveOperator(format!(
            "Choi operator is not Hermitian: defect {:?} > {:?}",
            defect, tol
        )));
    }
    let (vals, vecs) = choi
        .eigen_hermitian()
        .map_err(|e| QuantumError::CalculationError(format!("eigen: {:?}", e)))?;
    let vs = vecs.as_slice();

    let mut kraus = Vec::new();
    for (idx, lam) in vals.iter().enumerate() {
        let l = lam.re;
        if l < -tol {
            return Err(QuantumError::NonCptpChannel(format!(
                "Choi operator has a negative eigenvalue {:?}",
                l
            )));
        }
        if l <= tol {
            continue;
        }
        let scale = l.sqrt();
        // Eigenvector κ is column idx of V; its composite index is (i·d_out + j).
        let mut k_data = vec![Complex::new(R::zero(), R::zero()); d_out * d_in];
        for i in 0..d_in {
            for jj in 0..d_out {
                let v = vs[(i * d_out + jj) * d + idx];
                k_data[jj * d_in + i] = Complex::new(v.re * scale, v.im * scale);
            }
        }
        kraus.push(CausalTensor::from_slice(&k_data, &[d_out, d_in]));
    }
    if kraus.is_empty() {
        return Err(QuantumError::NonCptpChannel(
            "Choi operator has no positive spectrum (zero channel)".into(),
        ));
    }
    Ok(kraus)
}

/// Applies a channel to a state through its Kraus family: `E(ρ) = Σ K ρ Kᴴ`.
pub fn apply_kraus<R>(
    kraus: &[CausalTensor<Complex<R>>],
    rho: &CausalTensor<Complex<R>>,
) -> Result<CausalTensor<Complex<R>>, QuantumError>
where
    R: RealField + FromPrimitive + Default,
{
    if kraus.is_empty() {
        return Err(QuantumError::NonCptpChannel("empty Kraus family".into()));
    }
    let d_in = square_dim(rho)?;
    let shape = kraus[0].shape();
    if shape.len() != 2 || shape[1] != d_in {
        return Err(QuantumError::DimensionMismatch(format!(
            "Kraus shape {:?} incompatible with a {}x{} state",
            shape, d_in, d_in
        )));
    }
    let d_out = shape[0];
    if kraus.iter().any(|k| k.shape() != shape) {
        return Err(QuantumError::DimensionMismatch(
            "Kraus operators disagree on shape".into(),
        ));
    }
    let mut out = vec![Complex::new(R::zero(), R::zero()); d_out * d_out];
    for k in kraus {
        let t = k
            .matmul(rho)
            .and_then(|krho| krho.matmul(&k.dagger()?))
            .map_err(|e| QuantumError::CalculationError(format!("matmul: {:?}", e)))?;
        for (o, v) in out.iter_mut().zip(t.as_slice()) {
            *o = Complex::new(o.re + v.re, o.im + v.im);
        }
    }
    Ok(CausalTensor::from_slice(&out, &[d_out, d_out]))
}

/// Applies a channel to a state through its Choi operator:
/// `E(ρ)[j,l] = Σ_{ik} ρ[i,k] · J[(i,j),(k,l)]`.
pub fn apply_choi<R>(
    choi: &CausalTensor<Complex<R>>,
    rho: &CausalTensor<Complex<R>>,
    d_in: usize,
    d_out: usize,
) -> Result<CausalTensor<Complex<R>>, QuantumError>
where
    R: RealField,
{
    let d = square_dim(choi)?;
    if d != d_in * d_out || square_dim(rho)? != d_in {
        return Err(QuantumError::DimensionMismatch(format!(
            "Choi dim {} / state dim {} incompatible with ({}, {})",
            d,
            rho.shape()[0],
            d_in,
            d_out
        )));
    }
    let js = choi.as_slice();
    let rs = rho.as_slice();
    let mut out = vec![Complex::new(R::zero(), R::zero()); d_out * d_out];
    for jj in 0..d_out {
        for l in 0..d_out {
            let mut acc = Complex::new(R::zero(), R::zero());
            for i in 0..d_in {
                for k in 0..d_in {
                    let v = cmul(rs[i * d_in + k], js[(i * d_out + jj) * d + (k * d_out + l)]);
                    acc = Complex::new(acc.re + v.re, acc.im + v.im);
                }
            }
            out[jj * d_out + l] = acc;
        }
    }
    Ok(CausalTensor::from_slice(&out, &[d_out, d_out]))
}

/// Complete positivity check: `E` is CP iff its Choi operator is PSD
/// (spectrum ≥ −tol via the Hermitian eigendecomposition).
pub fn check_completely_positive<R>(
    choi: &CausalTensor<Complex<R>>,
    tol: R,
) -> Result<(), QuantumError>
where
    R: RealField + FromPrimitive + Default + core::fmt::Debug,
{
    validate_tolerance(tol)?;
    square_dim(choi)?;
    // PSD (hence CP) requires a finite Hermitian operator; eigen_hermitian would
    // otherwise silently certify the Hermitian part of a non-Hermitian input.
    if choi
        .as_slice()
        .iter()
        .any(|c| !c.re.is_finite() || !c.im.is_finite())
    {
        return Err(QuantumError::NonFiniteValue(
            "Choi operator contains a non-finite entry".into(),
        ));
    }
    let defect = hermiticity_defect(choi)?;
    if defect > tol {
        return Err(QuantumError::NonPositiveOperator(format!(
            "not completely positive: Choi is not Hermitian, defect {:?} > {:?}",
            defect, tol
        )));
    }
    let (vals, _) = choi
        .eigen_hermitian()
        .map_err(|e| QuantumError::CalculationError(format!("eigen: {:?}", e)))?;
    for lam in vals {
        if lam.re < -tol {
            return Err(QuantumError::NonCptpChannel(format!(
                "not completely positive: Choi eigenvalue {:?} < 0",
                lam.re
            )));
        }
    }
    Ok(())
}

/// Trace-preservation check: `E` is TP iff `Tr_out(J) = I_in`, computed with
/// the named-subset partial trace over the `out` leg.
pub fn check_trace_preserving<R>(
    choi: &CausalTensor<Complex<R>>,
    d_in: usize,
    d_out: usize,
    tol: R,
) -> Result<(), QuantumError>
where
    R: RealField + core::fmt::Debug,
{
    validate_tolerance(tol)?;
    let d = square_dim(choi)?;
    if d != d_in * d_out {
        return Err(QuantumError::DimensionMismatch(format!(
            "Choi dim {} != d_in·d_out = {}",
            d,
            d_in * d_out
        )));
    }
    let tr_out = partial_trace(choi, &[d_in, d_out], &[1])?;
    let id = identity_matrix::<R>(d_in);
    let defect = tr_out
        .as_slice()
        .iter()
        .zip(id.as_slice())
        .map(|(a, b)| {
            let dr = a.re - b.re;
            let di = a.im - b.im;
            (dr * dr + di * di).sqrt()
        })
        .fold(R::zero(), |acc, x| if x > acc { x } else { acc });
    if defect > tol {
        return Err(QuantumError::NonCptpChannel(format!(
            "not trace-preserving: max |Tr_out(J) − I| = {:?}",
            defect
        )));
    }
    Ok(())
}

/// The Choi operator of the identity channel on a `d`-dimensional space.
///
/// `J(id_d)[(i,j),(k,l)] = δ_ij·δ_kl`. Feeding it to [`apply_choi`] returns the
/// state unchanged, which is the defining property and what the tests assert.
///
/// It is the unit for [`choi_compose`] on both sides. With the two together
/// this module carries a category: dimensions are the objects, Choi operators
/// the morphisms, `choi_compose` the composition, and this the identity.
pub fn choi_identity<R>(d: usize) -> CausalTensor<Complex<R>>
where
    R: RealField,
{
    let n = d * d;
    let mut out = vec![Complex::new(R::zero(), R::zero()); n * n];
    for i in 0..d {
        for k in 0..d {
            // Row `(i, j)` with `j = i`, column `(k, l)` with `l = k`.
            out[(i * d + i) * n + (k * d + k)] = Complex::new(R::one(), R::zero());
        }
    }
    CausalTensor::from_slice(&out, &[n, n])
}

/// Composes two channels through their Choi operators.
///
/// `first` is `J(E)` for `E: L(H_a) → L(H_b)` and `then` is `J(F)` for
/// `F: L(H_b) → L(H_c)`. The result is `J(F∘E)`, the channel that applies `E`
/// and then `F`. The argument order is the order of application, not the order
/// of the `∘` symbol.
///
/// # The formula, and where it comes from
///
/// ```text
/// J(F∘E)[(a,c),(a',c')] = Σ_{b,b'} J(E)[(a,b),(a',b')] · J(F)[(b,c),(b',c')]
/// ```
///
/// A plain double contraction over the shared wire. No partial transpose and no
/// partial trace, both of which appear in the textbook form only because it
/// writes the contraction as a matrix product on the joint space.
///
/// It follows from this module's own [`apply_choi`] rather than from a
/// convention chosen to make it true. That function fixes
/// `E(ρ)[b,b'] = Σ_{a,a'} ρ[a,a']·J(E)[(a,b),(a',b')]`; substituting it into
/// itself for `F(E(ρ))` and collecting the coefficient of `ρ[a,a']` gives the
/// sum above.
///
/// # Composition needs no CPTP re-validation
///
/// This is linear-map composition. Complete positivity and trace preservation
/// are properties of the maps being composed and are inherited by the
/// composite, so a caller that validated its inputs does not have to validate
/// the result. The function is bound on `RealField` alone for that reason,
/// where the two `check_*` functions need more.
///
/// # Errors
///
/// [`QuantumError::DimensionMismatch`] if either operator is not square, if
/// `first` is not `d_a·d_b` square, or if `then` is not `d_b·d_c` square.
pub fn choi_compose<R>(
    first: &CausalTensor<Complex<R>>,
    then: &CausalTensor<Complex<R>>,
    d_a: usize,
    d_b: usize,
    d_c: usize,
) -> Result<CausalTensor<Complex<R>>, QuantumError>
where
    R: RealField,
{
    let de = square_dim(first)?;
    let df = square_dim(then)?;
    if de != d_a * d_b {
        return Err(QuantumError::DimensionMismatch(format!(
            "first Choi dim {} != d_a·d_b = {}",
            de,
            d_a * d_b
        )));
    }
    if df != d_b * d_c {
        return Err(QuantumError::DimensionMismatch(format!(
            "then Choi dim {} != d_b·d_c = {}",
            df,
            d_b * d_c
        )));
    }

    let es = first.as_slice();
    let fs = then.as_slice();
    let n = d_a * d_c;
    let mut out = vec![Complex::new(R::zero(), R::zero()); n * n];
    for a in 0..d_a {
        for c in 0..d_c {
            for ap in 0..d_a {
                for cp in 0..d_c {
                    let mut acc = Complex::new(R::zero(), R::zero());
                    for b in 0..d_b {
                        for bp in 0..d_b {
                            let v = cmul(
                                es[(a * d_b + b) * de + (ap * d_b + bp)],
                                fs[(b * d_c + c) * df + (bp * d_c + cp)],
                            );
                            acc = Complex::new(acc.re + v.re, acc.im + v.im);
                        }
                    }
                    out[(a * d_c + c) * n + (ap * d_c + cp)] = acc;
                }
            }
        }
    }
    Ok(CausalTensor::from_slice(&out, &[n, n]))
}
