/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Column means, the sample covariance matrix, and the conditional variance over one.
//!
//! A data matrix is passed as a row-major slice with its shape stated, the same way
//! [`fit_ridge`](crate::fit_ridge) takes a design: the crate's surface is over slices, and a
//! container that owns axes keeps the job of naming them.

use crate::errors::stats_error::StatsError;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_linear::{DenseMatrix, DenseVector, solve};
use deep_causality_num::FromPrimitive;

/// Validates a row-major data matrix and returns `(observations, variables)`.
///
/// The cell count is formed with `checked_mul` because both factors are the caller's and neither
/// is bounded by the slice: `observations · variables` for a stated shape wider than the address
/// space panics on the multiply in a debug build and wraps in a release one, and a wrapped product
/// that happens to match the data length would admit a shape whose indexing runs off the end.
fn shape<T>(data: &[T], observations: usize, variables: usize) -> Result<(), StatsError> {
    if observations == 0 || variables == 0 {
        return Err(StatsError::EmptyInput(
            "a data matrix needs at least one observation and one variable",
        ));
    }
    let cells = observations.checked_mul(variables).ok_or_else(|| {
        StatsError::DimensionMismatch(
            "the stated observation and variable counts overflow a usize: no matrix has that shape",
        )
    })?;
    if data.len() != cells {
        return Err(StatsError::DimensionMismatch(
            "the data length is not the product of the stated observation and variable counts",
        ));
    }
    Ok(())
}

/// The mean of every column of a row-major `observations × variables` matrix.
///
/// One [`mean`](crate::mean) per column would need the columns to exist as slices, which in a
/// row-major matrix they do not; this accumulates them in one pass over the rows instead. The
/// arithmetic is the same — a left-to-right sum divided by the count.
pub fn column_means<T>(
    data: &[T],
    observations: usize,
    variables: usize,
) -> Result<Vec<T>, StatsError>
where
    T: RealField + FromPrimitive,
{
    shape(data, observations, variables)?;
    let n = T::from_usize(observations).ok_or_else(|| {
        StatsError::ConversionFailed(
            "an observation count is not representable in the working scalar",
        )
    })?;

    let mut means = vec![T::zero(); variables];
    for row in 0..observations {
        let base = row * variables;
        for (col, mean) in means.iter_mut().enumerate() {
            *mean += data[base + col];
        }
    }
    for mean in means.iter_mut() {
        *mean /= n;
    }
    Ok(means)
}

/// The sample covariance matrix, with Bessel's correction.
///
/// Returns the `variables × variables` matrix whose `(i, j)` entry is
/// `Σ_r (x_ri − μ_i)(x_rj − μ_j) / (n − 1)`, row-major.
///
/// Two passes, deliberately, for the reason [`variance`](crate::variance) takes two: forming the
/// means first keeps `Σx_i x_j` out of the computation, so a small covariance around large means
/// survives the cancellation a one-pass `Σx_i x_j − n μ_i μ_j` would lose it to.
pub fn covariance_matrix<T>(
    data: &[T],
    observations: usize,
    variables: usize,
) -> Result<Vec<T>, StatsError>
where
    T: RealField + FromPrimitive,
{
    shape(data, observations, variables)?;
    if observations < 2 {
        return Err(StatsError::InsufficientSamples(
            "the corrected covariance needs two observations: with one the divisor n − 1 is zero",
        ));
    }
    let means = column_means(data, observations, variables)?;
    let denominator = T::from_usize(observations - 1).ok_or_else(|| {
        StatsError::ConversionFailed(
            "an observation count is not representable in the working scalar",
        )
    })?;

    let mut cov = vec![T::zero(); variables * variables];
    for row in 0..observations {
        let base = row * variables;
        for i in 0..variables {
            let di = data[base + i] - means[i];
            for j in 0..variables {
                cov[i * variables + j] += di * (data[base + j] - means[j]);
            }
        }
    }
    for entry in cov.iter_mut() {
        *entry /= denominator;
    }
    Ok(cov)
}

/// The variance of one variable given a set of others, over a covariance matrix.
///
/// The Schur complement `Σ_yy − Σ_yP (Σ_PP + λI)⁻¹ Σ_Py`: the variance left in the target once its
/// parents have explained what they can, which is the residual variance of regressing the target on
/// them. An empty parent set leaves the marginal variance `Σ_yy`.
///
/// `ridge` is added to the parent block's diagonal so the solve stays defined when the parents are
/// collinear. It is the caller's, not a default: a ridge large enough to matter also biases the
/// result toward the marginal variance.
///
/// # The parents are a set
///
/// A repeated parent, and the target among its own parents, are both refused. Neither is a
/// harmless spelling of a smaller set. A parent listed twice puts a duplicated row and column into
/// `Σ_PP`, which is exactly singular, so at `ridge = 0` it is refused as collinear and at any
/// positive ridge it is *solvable* and answers something other than the same parent listed once —
/// so the number returned would depend on how many times the caller named a variable rather than
/// on which variables were named. Conditioning the target on itself is degenerate in the same way:
/// the residual is zero by definition, and at a positive ridge this returns a strictly positive
/// number instead. Both are caller errors that an index check alone cannot see.
///
/// # Errors
///
/// [`StatsError::DimensionMismatch`] when an index lies outside the matrix, when a parent is
/// repeated, or when the target is among the parents.
///
/// [`StatsError::RankDeficient`] when the parent block cannot be solved at the ridge given — which
/// at `ridge = 0` means exactly collinear parents. That is a refusal where an implementation that
/// floored the pivot would return a number, and the number would be fiction.
pub fn conditional_variance<T>(
    covariance: &[T],
    variables: usize,
    target: usize,
    parents: &[usize],
    ridge: T,
) -> Result<T, StatsError>
where
    T: RealField + FromPrimitive,
{
    shape(covariance, variables, variables)?;
    if target >= variables || parents.iter().any(|&p| p >= variables) {
        return Err(StatsError::DimensionMismatch(
            "a target or parent index lies outside the covariance matrix",
        ));
    }
    if parents.contains(&target) {
        return Err(StatsError::DimensionMismatch(
            "the target is among its own parents: the variance of a variable given itself is zero \
             by definition, and a positive ridge would return a positive number for it",
        ));
    }
    // Quadratic in the parent count, which is the number of parents of one node in a causal graph.
    // A set would allocate to say the same thing.
    if parents
        .iter()
        .enumerate()
        .any(|(i, p)| parents[..i].contains(p))
    {
        return Err(StatsError::DimensionMismatch(
            "a parent is listed twice: the parent block is then singular, and at a positive ridge \
             the answer would depend on how many times a variable was named",
        ));
    }
    let sigma_yy = covariance[target * variables + target];

    let k = parents.len();
    if k == 0 {
        return Ok(sigma_yy);
    }

    let mut sigma_yp = vec![T::zero(); k];
    for (slot, &p) in sigma_yp.iter_mut().zip(parents.iter()) {
        *slot = covariance[target * variables + p];
    }

    let mut sigma_pp = vec![T::zero(); k * k];
    for (i, &pi) in parents.iter().enumerate() {
        for (j, &pj) in parents.iter().enumerate() {
            let mut v = covariance[pi * variables + pj];
            if i == j {
                v += ridge;
            }
            sigma_pp[i * k + j] = v;
        }
    }

    // Through `deep_causality_linear`, as the ridge fit is. LU with partial pivoting rather than a
    // Cholesky: the ridged parent block is positive definite for a positive ridge, but at a zero
    // ridge on collinear parents it is only semi-definite, and a Cholesky that floored the pivot
    // would complete and return a plausible number where there is none.
    let block = DenseMatrix::from_vec(sigma_pp, k, k)
        .map_err(|_| StatsError::DimensionMismatch("the parent block is not square"))?;
    let z = solve(&block, &DenseVector::from_vec(sigma_yp.clone())).map_err(|_| {
        StatsError::RankDeficient(
            "the parent block has no unique solution at this ridge: the parents are collinear",
        )
    })?;

    let mut reduction = T::zero();
    for (i, zi) in z.as_slice().iter().enumerate() {
        reduction += sigma_yp[i] * *zi;
    }
    Ok(sigma_yy - reduction)
}
