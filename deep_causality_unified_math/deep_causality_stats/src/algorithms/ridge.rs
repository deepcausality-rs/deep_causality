/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Ridge-penalised least squares, in a materialised and a streaming form.

use crate::errors::stats_error::StatsError;
use crate::types::ridge_fit::RidgeFit;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::RealField;
use deep_causality_linear::{DenseMatrix, DenseVector, solve};
use deep_causality_num::FromPrimitive;

/// Fits `β = (XᵀX + λI)⁻¹ Xᵀy` over a materialised design.
///
/// At `λ = 0` a rank-deficient design has no unique solution and is refused. The penalty is what
/// makes the normal equations invertible, so a caller that supplies none is asking for a solution
/// that may not exist.
pub fn fit_ridge<T>(
    x: &[alloc::vec::Vec<T>],
    y: &[T],
    penalty: T,
) -> Result<RidgeFit<T>, StatsError>
where
    T: RealField + FromPrimitive,
{
    if x.is_empty() {
        return Err(StatsError::EmptyInput("a fit needs at least one row"));
    }
    if y.len() != x.len() {
        return Err(StatsError::DimensionMismatch(
            "the response has a different length from the design",
        ));
    }
    // The width is taken from the first row and validated by the shared body, which refuses a
    // zero width and checks every row against it. Repeating either check here would only add a
    // second wording for the same refusal.
    accumulate_and_solve(
        x.iter().map(|r| r.as_slice()).zip(y.iter().copied()),
        penalty,
        x[0].len(),
    )
}

/// The shared body: accumulate `XᵀX + λI` and `Xᵀy` from rows, then solve.
fn accumulate_and_solve<'a, T, I>(
    rows: I,
    penalty: T,
    columns: usize,
) -> Result<RidgeFit<T>, StatsError>
where
    T: RealField + FromPrimitive,
    I: Iterator<Item = (&'a [T], T)>,
    T: 'a,
{
    if columns == 0 {
        return Err(StatsError::DimensionMismatch(
            "a design with no columns has nothing to fit",
        ));
    }

    if !penalty.is_finite() {
        return Err(StatsError::NonFiniteInput(
            "a non-finite penalty has no fit: an infinite one drives every coefficient to zero \
             and would return that as an estimate",
        ));
    }

    // A negative penalty is not rejected here. It subtracts from the diagonal rather than adding
    // to it, which is still a solvable system while the diagonal survives; only a penalty that
    // cancels the design leaves no unique solution, and the vanishing pivot below reports that.
    let p = columns;
    let mut xtx = vec![T::zero(); p * p];
    let mut xty = vec![T::zero(); p];
    let mut kept: Vec<(Vec<T>, T)> = Vec::new();

    for (row, yi) in rows {
        if row.len() != p {
            return Err(StatsError::DimensionMismatch(
                "every design row must carry the same number of columns",
            ));
        }
        if row.iter().any(|v| !v.is_finite()) || !yi.is_finite() {
            return Err(StatsError::NonFiniteInput(
                "a non-finite observation has no least-squares fit",
            ));
        }
        for a in 0..p {
            xty[a] += row[a] * yi;
            let ra = row[a];
            for b in 0..p {
                xtx[a * p + b] += ra * row[b];
            }
        }
        kept.push((row.to_vec(), yi));
    }

    let n = kept.len();
    if n == 0 {
        return Err(StatsError::EmptyInput("a fit needs at least one row"));
    }
    for a in 0..p {
        xtx[a * p + a] += penalty;
    }

    // The solve goes through `deep_causality_linear` rather than a local elimination. `xtx` is
    // already the row-major square layout `DenseMatrix` takes, so the crossing costs nothing.
    //
    // LU with partial pivoting rather than a Cholesky: the penalised normal matrix is positive
    // definite for a positive penalty, but at a zero penalty on a rank-deficient design it is only
    // positive semi-definite, and a Cholesky that floored a non-positive pivot would return a
    // plausible answer where there is none. A vanishing pivot is reported instead. A negative
    // penalty leaves the matrix indefinite, which the same factorisation handles and Cholesky
    // does not.
    let normal = DenseMatrix::from_vec(xtx, p, p)
        .map_err(|_| StatsError::DimensionMismatch("the normal matrix is not square"))?;
    let beta = solve(&normal, &DenseVector::from_vec(xty))
        .map_err(|_| {
            StatsError::RankDeficient(
                "the design has no unique solution at this penalty: a pivot vanished",
            )
        })?
        .as_slice()
        .to_vec();
    if beta.iter().any(|v| !v.is_finite()) {
        return Err(StatsError::RankDeficient(
            "the solve produced a non-finite coefficient: the design is singular at this penalty",
        ));
    }

    // Residual variance on `max(n − p, 1)` degrees of freedom.
    let mut rss = T::zero();
    for (row, yi) in &kept {
        let mut fitted = T::zero();
        for (a, &r) in row.iter().enumerate() {
            fitted += beta[a] * r;
        }
        let e = *yi - fitted;
        rss += e * e;
    }
    let dof = T::from_usize(if n > p { n - p } else { 1 }).ok_or_else(|| {
        StatsError::ConversionFailed(
            "a degrees-of-freedom count is not representable in the working scalar",
        )
    })?;
    Ok(RidgeFit {
        beta,
        sigma2: rss / dof,
    })
}

/// Fits the same model without materialising the design.
///
/// Accumulates `XᵀX` and `Xᵀy` from rows supplied one at a time, for a caller whose design is
/// larger than it wants in memory or is filtered as it goes. Over the same rows the result agrees
/// with [`fit_ridge`] to the precision in use.
pub fn fit_ridge_streaming<T, I>(
    rows: I,
    penalty: T,
    columns: usize,
) -> Result<RidgeFit<T>, StatsError>
where
    T: RealField + FromPrimitive,
    I: IntoIterator<Item = (Vec<T>, T)>,
{
    let materialised: Vec<(Vec<T>, T)> = rows.into_iter().collect();
    accumulate_and_solve(
        materialised.iter().map(|(r, y)| (r.as_slice(), *y)),
        penalty,
        columns,
    )
}
