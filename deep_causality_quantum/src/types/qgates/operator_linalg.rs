/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Operator-layer linear algebra on `CausalTensor<Complex<R>>` matrices:
//! trace, norms, the partial trace `Tr_B` (named-subset generalization), the
//! identity embedding onto a leg union, and the operator commutator.
//!
//! These are the L0/L2/L4 rungs of the operator build ladder (design B4):
//! everything is plain index arithmetic over the existing tensor substrate;
//! no new numeric substance.

use crate::QuantumError;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, Tensor};

/// Validates that `op` is a non-empty square matrix and returns its dimension.
pub(crate) fn square_dim<R>(op: &CausalTensor<Complex<R>>) -> Result<usize, QuantumError>
where
    R: RealField,
{
    let shape = op.shape();
    if shape.len() != 2 {
        return Err(QuantumError::DimensionMismatch(format!(
            "expected a matrix, got rank {}",
            shape.len()
        )));
    }
    if shape[0] != shape[1] {
        return Err(QuantumError::DimensionMismatch(format!(
            "expected a square matrix, got {}x{}",
            shape[0], shape[1]
        )));
    }
    if shape[0] == 0 {
        return Err(QuantumError::DimensionMismatch("empty matrix".into()));
    }
    Ok(shape[0])
}

/// The matrix trace `Tr(M)`.
pub fn matrix_trace<R>(op: &CausalTensor<Complex<R>>) -> Result<Complex<R>, QuantumError>
where
    R: RealField,
{
    let d = square_dim(op)?;
    let s = op.as_slice();
    let mut tr = Complex::new(R::zero(), R::zero());
    for i in 0..d {
        let c = s[i * d + i];
        tr = Complex::new(tr.re + c.re, tr.im + c.im);
    }
    Ok(tr)
}

/// The Frobenius norm `‖M‖_F = √(Σ |m_ij|²)`.
pub fn frobenius_norm<R>(op: &CausalTensor<Complex<R>>) -> R
where
    R: RealField,
{
    op.as_slice()
        .iter()
        .fold(R::zero(), |acc, c| acc + c.re * c.re + c.im * c.im)
        .sqrt()
}

/// The Hermiticity defect `max_ij |M_ij − conj(M_ji)|` (zero iff `M = Mᴴ`).
pub fn hermiticity_defect<R>(op: &CausalTensor<Complex<R>>) -> Result<R, QuantumError>
where
    R: RealField,
{
    let d = square_dim(op)?;
    let s = op.as_slice();
    let mut max = R::zero();
    for i in 0..d {
        for j in 0..d {
            let a = s[i * d + j];
            let b = s[j * d + i];
            let dr = a.re - b.re;
            let di = a.im + b.im;
            let m = (dr * dr + di * di).sqrt();
            if m > max {
                max = m;
            }
        }
    }
    Ok(max)
}

/// The `d×d` complex identity matrix.
pub fn identity_matrix<R>(d: usize) -> CausalTensor<Complex<R>>
where
    R: RealField,
{
    let mut data = vec![Complex::new(R::zero(), R::zero()); d * d];
    for i in 0..d {
        data[i * d + i] = Complex::new(R::one(), R::zero());
    }
    CausalTensor::from_slice(&data, &[d, d])
}

/// The operator commutator `[A, B] = A·B − B·A` on complex matrices.
pub fn matrix_commutator<R>(
    a: &CausalTensor<Complex<R>>,
    b: &CausalTensor<Complex<R>>,
) -> Result<CausalTensor<Complex<R>>, QuantumError>
where
    R: RealField + FromPrimitive + Default,
{
    let da = square_dim(a)?;
    let db = square_dim(b)?;
    if da != db {
        return Err(QuantumError::DimensionMismatch(format!(
            "commutator operands differ: {}x{} vs {}x{}",
            da, da, db, db
        )));
    }
    let ab = a
        .matmul(b)
        .map_err(|e| QuantumError::CalculationError(format!("matmul: {:?}", e)))?;
    let ba = b
        .matmul(a)
        .map_err(|e| QuantumError::CalculationError(format!("matmul: {:?}", e)))?;
    Ok(ab - ba)
}

/// Whether two Hilbert supports (sets of leg indices) intersect — the freeze
/// check computes a commutator only for intersecting supports.
pub fn supports_intersect(a: &BTreeSet<usize>, b: &BTreeSet<usize>) -> bool {
    a.intersection(b).next().is_some()
}

/// The partial trace over a named subset of tensor legs.
///
/// `op` is a square matrix on the composite space `⊗_i H_i` with per-leg
/// dimensions `dims` (row-major: leg 0 most significant, `∏ dims == matrix
/// dim`); `traced` names the legs to trace out. The classic `Tr_B` on
/// `H_A ⊗ H_B` is `partial_trace(op, &[d_a, d_b], &[1])`.
///
/// # This does not preserve commutation
///
/// Partial trace is positive and linear. It is **not** an algebra homomorphism,
/// and in particular `[X, Y] = 0` does not imply `[Tr_B X, Tr_B Y] = 0`. The
/// counterexample is proved in Lean as `quantum.partial_trace_nonpreservation`,
/// closed by `decide` over ℤ: two commuting operators whose partial traces have
/// commutator `[[0, 4], [−4, 0]]`. Its Rust witness is
/// `formalization_lean::partial_trace_tests::test_partial_trace_nonpreservation_counterexample`.
///
/// The consequence for a caller is concrete. Marginalising a factorization that
/// `validate` has certified can destroy the Markov property that certificate
/// stands for, and nothing at this call site will say so. The roadmap item
/// `quantum.partial_trace_preservation` states the unconditional version and is
/// **false**.
///
/// The sound path is [`partial_trace_preservation_boundary`]: a boundary
/// operator of the form `Z ⊗ 1_B` that commutes with `M` does force `Z` to
/// commute with `Tr_B(M)`. A caller that needs commutation to survive
/// marginalisation should call that; this function will not check it.
///
/// See `deep_causality_quantum/LEAN_QUANTUM.md` for both statements.
///
/// # Errors
/// Rejects a non-square operator, a `dims` product that disagrees with the
/// matrix dimension, and duplicate/out-of-range `traced` legs — all as
/// [`QuantumError::PartialTraceShape`] (shape errors) or
/// [`QuantumError::DimensionMismatch`].
pub fn partial_trace<R>(
    op: &CausalTensor<Complex<R>>,
    dims: &[usize],
    traced: &[usize],
) -> Result<CausalTensor<Complex<R>>, QuantumError>
where
    R: RealField,
{
    let d = square_dim(op)?;
    // Checked product: a caller-supplied `dims` can overflow usize (debug panic /
    // release wrap) before this shape gate would otherwise fire.
    let prod = dims.iter().try_fold(1usize, |acc, &x| acc.checked_mul(x));
    if prod != Some(d) || dims.contains(&0) {
        return Err(QuantumError::PartialTraceShape(format!(
            "dims {:?} do not factor the {}x{} operator",
            dims, d, d
        )));
    }
    let traced_set: BTreeSet<usize> = traced.iter().copied().collect();
    if traced_set.len() != traced.len() || traced.iter().any(|&t| t >= dims.len()) {
        return Err(QuantumError::PartialTraceShape(format!(
            "traced legs {:?} must be unique and < {}",
            traced,
            dims.len()
        )));
    }

    let kept: Vec<usize> = (0..dims.len())
        .filter(|i| !traced_set.contains(i))
        .collect();
    let d_keep: usize = kept.iter().map(|&i| dims[i]).product();
    let d_tr: usize = traced_set.iter().map(|&i| dims[i]).product();

    // Row-major strides of each leg inside the composite index.
    let mut strides = vec![1usize; dims.len()];
    for i in (0..dims.len().saturating_sub(1)).rev() {
        strides[i] = strides[i + 1] * dims[i + 1];
    }

    // Composite offset of a keep-side (or traced-side) multi-index.
    let offset = |legs: &[usize], mut idx: usize| -> usize {
        let mut off = 0usize;
        for &leg in legs.iter().rev() {
            let dim = dims[leg];
            off += (idx % dim) * strides[leg];
            idx /= dim;
        }
        off
    };
    let traced_legs: Vec<usize> = traced_set.into_iter().collect();

    let s = op.as_slice();
    let mut out = vec![Complex::new(R::zero(), R::zero()); d_keep * d_keep];
    for rk in 0..d_keep {
        let row_base = offset(&kept, rk);
        for ck in 0..d_keep {
            let col_base = offset(&kept, ck);
            let mut acc = Complex::new(R::zero(), R::zero());
            for t in 0..d_tr {
                let t_off = offset(&traced_legs, t);
                let c = s[(row_base + t_off) * d + (col_base + t_off)];
                acc = Complex::new(acc.re + c.re, acc.im + c.im);
            }
            out[rk * d_keep + ck] = acc;
        }
    }
    Ok(CausalTensor::from_slice(&out, &[d_keep, d_keep]))
}

/// Embeds an operator acting on the legs `op_legs` into the full `space`
/// (leg → dimension, ascending leg order = factor order), acting as the
/// identity on every other leg — the Kronecker-with-identity alignment the
/// freeze commutator uses to compare factors on their support union.
///
/// # Errors
/// Rejects an operator whose dimension disagrees with `∏ dims(op_legs)`, and
/// `op_legs` not contained in `space`.
pub fn embed_on_legs<R>(
    op: &CausalTensor<Complex<R>>,
    op_legs: &BTreeSet<usize>,
    space: &BTreeMap<usize, usize>,
) -> Result<CausalTensor<Complex<R>>, QuantumError>
where
    R: RealField,
{
    let d_op = square_dim(op)?;
    if !op_legs.iter().all(|l| space.contains_key(l)) {
        return Err(QuantumError::DimensionMismatch(format!(
            "operator legs {:?} not contained in the space {:?}",
            op_legs,
            space.keys().collect::<Vec<_>>()
        )));
    }
    let expect = op_legs
        .iter()
        .try_fold(1usize, |acc, l| acc.checked_mul(space[l]))
        .ok_or_else(|| {
            QuantumError::DimensionMismatch(format!(
                "operator legs {:?} dimension product overflows usize",
                op_legs
            ))
        })?;
    if expect != d_op {
        return Err(QuantumError::DimensionMismatch(format!(
            "operator dim {} does not match its legs' product {}",
            d_op, expect
        )));
    }

    let legs: Vec<usize> = space.keys().copied().collect();
    let dims: Vec<usize> = legs.iter().map(|l| space[l]).collect();
    let d_full = dims
        .iter()
        .try_fold(1usize, |acc, &x| acc.checked_mul(x))
        .ok_or_else(|| {
            QuantumError::DimensionMismatch(format!(
                "space dimension product {:?} overflows usize",
                dims
            ))
        })?;

    // Row-major strides per position in the full space.
    let mut strides = vec![1usize; dims.len()];
    for i in (0..dims.len().saturating_sub(1)).rev() {
        strides[i] = strides[i + 1] * dims[i + 1];
    }
    let op_pos: Vec<usize> = legs
        .iter()
        .enumerate()
        .filter(|(_, l)| op_legs.contains(l))
        .map(|(p, _)| p)
        .collect();
    let rest_pos: Vec<usize> = legs
        .iter()
        .enumerate()
        .filter(|(_, l)| !op_legs.contains(l))
        .map(|(p, _)| p)
        .collect();
    let d_rest: usize = rest_pos.iter().map(|&p| dims[p]).product();

    let offset = |pos: &[usize], mut idx: usize| -> usize {
        let mut off = 0usize;
        for &p in pos.iter().rev() {
            let dim = dims[p];
            off += (idx % dim) * strides[p];
            idx /= dim;
        }
        off
    };

    let s = op.as_slice();
    let alloc = d_full.checked_mul(d_full).ok_or_else(|| {
        QuantumError::DimensionMismatch(format!(
            "embedded operator size {}² overflows usize",
            d_full
        ))
    })?;
    let mut out = vec![Complex::new(R::zero(), R::zero()); alloc];
    for r_op in 0..d_op {
        let row_op = offset(&op_pos, r_op);
        for c_op in 0..d_op {
            let col_op = offset(&op_pos, c_op);
            let v = s[r_op * d_op + c_op];
            for t in 0..d_rest {
                let t_off = offset(&rest_pos, t);
                out[(row_op + t_off) * d_full + (col_op + t_off)] = v;
            }
        }
    }
    Ok(CausalTensor::from_slice(&out, &[d_full, d_full]))
}

/// What a boundary check certifies: the measured hypothesis and the bound it buys.
///
/// Returned by [`partial_trace_preservation_boundary`]. The fields are separate because the
/// decision and the number are separate questions: `holds` answers "did the hypothesis pass at the
/// tolerance I named", and `conclusion_bound` answers "what may I then assert about the traced
/// commutator". A caller composing several of these needs the second.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundaryWarrant<R> {
    /// The measured `‖[Z ⊗ 1_B, M]‖_F`.
    pub hypothesis_residual: R,
    /// The tolerance the caller named for that residual.
    pub tolerance: R,
    /// `√(d_B)`, the factor by which the partial trace can amplify a residual.
    pub amplification: R,
    /// The certified bound on `‖[Z, Tr_B(M)]‖_F`, namely `√(d_B) · hypothesis_residual`.
    pub conclusion_bound: R,
    /// Whether `hypothesis_residual ≤ tolerance`.
    pub holds: bool,
}

/// The sound path past [`partial_trace`]'s non-preservation: given `Z` on the kept factor and `M`
/// on the whole space, measure how nearly `Z ⊗ 1_B` commutes with `M`, and return the bound that
/// buys on `[Z, Tr_B(M)]`.
///
/// # Why this returns a bound rather than a boolean
///
/// The Lean theorem `quantum.partial_trace_preservation_boundary` concludes
/// `Z · Tr_B(M) = Tr_B(M) · Z` from the hypothesis `(Z ⊗ 1_B) · M = M · (Z ⊗ 1_B)`, and that
/// hypothesis is **propositional equality over a general `CommRing`**. It carries no epsilon. A
/// caller working in floating point can only ever measure the hypothesis to a tolerance, so
/// invoking the theorem on the strength of a numeric check would substitute an approximate premise
/// into an exact-hypothesis result. That substitution is not sound, and this function does not make
/// it.
///
/// What is sound is the *unconditional* transport identity, `quantum.partial_trace.commutator_transport`:
///
/// ```text
/// Tr_B([Z ⊗ 1_B, M]) = [Z, Tr_B(M)]
/// ```
///
/// an equality with no hypothesis at all. Combined with the contraction
/// `‖Tr_B(E)‖_F ≤ √(d_B) · ‖E‖_F`, which follows from Cauchy-Schwarz on the traced index and is
/// tight at `E = F ⊗ 1_B`, a residual of `ε` in the hypothesis certifies
/// `‖[Z, Tr_B(M)]‖_F ≤ √(d_B) · ε` in the conclusion. Exactly zero in, exactly zero out, which
/// recovers the Lean theorem as the vanishing case; anything else in, an amplified bound out.
///
/// **So the ruling this settles is: a tolerance-satisfied commutator is not warrant for the exact
/// conclusion, and is warrant for the conclusion at `√(d_B)` times the tolerance.** The
/// amplification is the price of marginalising, and it grows with the dimension traced away.
///
/// # The form is built, not checked
///
/// The theorem's other hypothesis, that the operator has the shape `Z ⊗ 1_B`, is decidable from leg
/// data alone. Rather than check it, this function constructs `Z ⊗ 1_B` from `z`, so the form holds
/// by construction and cannot be got wrong by a caller.
///
/// # Errors
///
/// [`QuantumError::PartialTraceShape`] if `dims` does not factor `m`, and
/// [`QuantumError::DimensionMismatch`] if `z` is not square of side `dims[0]`. The kept factor is
/// `dims[0]` and the traced factor `dims[1]`, matching [`partial_trace`] with `traced = &[1]`.
pub fn partial_trace_preservation_boundary<R>(
    z: &CausalTensor<Complex<R>>,
    m: &CausalTensor<Complex<R>>,
    dims: [usize; 2],
    tolerance: R,
) -> Result<BoundaryWarrant<R>, QuantumError>
where
    R: RealField + FromPrimitive + Default,
{
    let [d_a, d_b] = dims;
    let full = square_dim(m)?;
    if d_a.checked_mul(d_b) != Some(full) || d_a == 0 || d_b == 0 {
        return Err(QuantumError::PartialTraceShape(format!(
            "dims {:?} do not factor the {}x{} operator",
            dims, full, full
        )));
    }
    if square_dim(z)? != d_a {
        return Err(QuantumError::DimensionMismatch(format!(
            "the kept-factor operator must be {}x{} to match dims {:?}",
            d_a, d_a, dims
        )));
    }

    // `Z ⊗ 1_B`, so the boundary form is a construction rather than a claim.
    let boundary = z
        .kronecker(&identity_matrix::<R>(d_b))
        .map_err(|e| QuantumError::DimensionMismatch(format!("{e}")))?;

    let hypothesis_residual = frobenius_norm(&matrix_commutator(&boundary, m)?);
    let amplification = R::from_f64((d_b as f64).sqrt()).ok_or_else(|| {
        QuantumError::CalculationError("scalar type cannot represent √(d_B)".into())
    })?;

    Ok(BoundaryWarrant {
        conclusion_bound: amplification * hypothesis_residual,
        holds: hypothesis_residual <= tolerance,
        hypothesis_residual,
        tolerance,
        amplification,
    })
}
