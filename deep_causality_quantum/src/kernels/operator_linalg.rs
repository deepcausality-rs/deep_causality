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
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;
use deep_causality_num_complex::Complex;
use deep_causality_tensor::{CausalTensor, Tensor};
use std::collections::{BTreeMap, BTreeSet};

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
    let prod: usize = dims.iter().product();
    if prod != d || dims.contains(&0) {
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

    let kept: Vec<usize> = (0..dims.len()).filter(|i| !traced_set.contains(i)).collect();
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
    let expect: usize = op_legs.iter().map(|l| space[l]).product();
    if expect != d_op {
        return Err(QuantumError::DimensionMismatch(format!(
            "operator dim {} does not match its legs' product {}",
            d_op, expect
        )));
    }

    let legs: Vec<usize> = space.keys().copied().collect();
    let dims: Vec<usize> = legs.iter().map(|l| space[l]).collect();
    let d_full: usize = dims.iter().product();

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
    let mut out = vec![Complex::new(R::zero(), R::zero()); d_full * d_full];
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
