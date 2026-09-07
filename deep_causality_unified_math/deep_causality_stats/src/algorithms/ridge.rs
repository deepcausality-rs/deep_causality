/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Ridge-penalised least squares, in a materialised and a streaming form.

use crate::errors::stats_error::StatsError;
use crate::types::ridge_config::RidgeConfig;
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
    config: &RidgeConfig<T>,
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
    //
    // The rows are handed over as a borrowing iterator rather than copied: the caller already owns
    // the design, and the shared body reads it twice rather than keeping a copy of its own.
    accumulate_and_solve(
        || x.iter().map(|r| r.as_slice()).zip(y.iter().copied()),
        config,
        x[0].len(),
    )
}

/// The shared body: accumulate `XᵀX + λI` and `Xᵀy` from rows, then solve.
///
/// # Two passes, and no copy
///
/// Takes a *factory* rather than an iterator, because the residual sum of squares needs `β`, `β`
/// needs the whole accumulation, and so the rows have to be read a second time. An earlier version
/// took a once-consumable iterator and paid for that by `collect`ing every row into a `Vec` — which
/// made a function documented as being "for a caller whose design is larger than it wants in
/// memory" hold the entire design, twice over, and made the materialised `fit_ridge` copy a design
/// its caller already owned.
///
/// The alternative is the algebraic residual `Σy² − 2β·Xᵀy + βᵀXᵀXβ`, which needs one pass and
/// `O(p²)` memory. It is not used here: it subtracts quantities of similar size, and loses to
/// cancellation exactly where the fit is good and `RSS` is small — the same reason `variance`
/// forms its deviations in two passes instead of taking `Σx² − nx̄²`.
fn accumulate_and_solve<'a, T, I, F>(
    rows: F,
    config: &RidgeConfig<T>,
    columns: usize,
) -> Result<RidgeFit<T>, StatsError>
where
    T: RealField + FromPrimitive + 'a,
    I: Iterator<Item = (&'a [T], T)>,
    F: Fn() -> I,
{
    if columns == 0 {
        return Err(StatsError::DimensionMismatch(
            "a design with no columns has nothing to fit",
        ));
    }

    if !config.penalty.is_finite() {
        return Err(StatsError::NonFiniteInput(
            "a non-finite penalty has no fit: an infinite one drives every coefficient to zero \
             and would return that as an estimate",
        ));
    }

    let p = columns;
    if !config.penalisation.fits_width(p) {
        return Err(StatsError::DimensionMismatch(
            "the column exempted from the penalty lies outside the design",
        ));
    }
    let exempt = config.penalisation.exempt();

    let mut xtx = vec![T::zero(); p * p];
    let mut xty = vec![T::zero(); p];
    let mut n = 0usize;

    for (row, yi) in rows() {
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
        n += 1;
    }

    if n == 0 {
        return Err(StatsError::EmptyInput("a fit needs at least one row"));
    }

    // A negative penalty is not rejected here. It subtracts from the diagonal rather than adding
    // to it, which is still a solvable system while the diagonal survives; only a penalty that
    // cancels the design leaves no unique solution, and the vanishing pivot below reports that.
    for a in 0..p {
        if exempt != Some(a) {
            xtx[a * p + a] += config.penalty;
        }
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

    // Residual variance on `max(n − p, 1)` degrees of freedom. The second pass over the rows.
    let mut rss = T::zero();
    for (row, yi) in rows() {
        let mut fitted = T::zero();
        for (a, &r) in row.iter().enumerate() {
            fitted += beta[a] * r;
        }
        let e = yi - fitted;
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
/// For a caller whose design is larger than it wants in memory, or is filtered as it goes: the row
/// source is re-iterated rather than stored, so the memory held is `O(p²)` for the normal matrix
/// regardless of how many rows pass through. Over the same rows the result agrees with [`fit_ridge`]
/// to the precision in use.
///
/// # Why the source must be re-iterable
///
/// The fit needs two looks at the data — one to accumulate `XᵀX` and `Xᵀy`, one to form the
/// residuals once `β` is known — so the bound is `Clone` on the iterator rather than a plain
/// `IntoIterator`. An earlier signature took a once-consumable iterator and bought the second pass
/// by `collect`ing the whole design into a `Vec`, which is precisely the memory this function
/// exists to avoid: it made "streaming" hold every row, and then `accumulate_and_solve` held a
/// second copy on top.
///
/// A caller that owns its rows already should call [`fit_ridge`], which borrows them.
pub fn fit_ridge_streaming<T, I>(
    rows: I,
    config: &RidgeConfig<T>,
    columns: usize,
) -> Result<RidgeFit<T>, StatsError>
where
    T: RealField + FromPrimitive,
    I: IntoIterator<Item = (Vec<T>, T)> + Clone,
{
    // Each pass rebuilds its rows from the caller's source. The `Vec` a row arrives in is dropped
    // as soon as it has been accumulated, so the peak is one row, not the design.
    let mut xtx = vec![T::zero(); columns * columns];
    let mut xty = vec![T::zero(); columns];
    let mut n = 0usize;

    if columns == 0 {
        return Err(StatsError::DimensionMismatch(
            "a design with no columns has nothing to fit",
        ));
    }
    if !config.penalty.is_finite() {
        return Err(StatsError::NonFiniteInput(
            "a non-finite penalty has no fit: an infinite one drives every coefficient to zero \
             and would return that as an estimate",
        ));
    }
    if !config.penalisation.fits_width(columns) {
        return Err(StatsError::DimensionMismatch(
            "the column exempted from the penalty lies outside the design",
        ));
    }
    let exempt = config.penalisation.exempt();

    for (row, yi) in rows.clone() {
        if row.len() != columns {
            return Err(StatsError::DimensionMismatch(
                "every design row must carry the same number of columns",
            ));
        }
        if row.iter().any(|v| !v.is_finite()) || !yi.is_finite() {
            return Err(StatsError::NonFiniteInput(
                "a non-finite observation has no least-squares fit",
            ));
        }
        for a in 0..columns {
            xty[a] += row[a] * yi;
            let ra = row[a];
            for b in 0..columns {
                xtx[a * columns + b] += ra * row[b];
            }
        }
        n += 1;
    }
    if n == 0 {
        return Err(StatsError::EmptyInput("a fit needs at least one row"));
    }
    for a in 0..columns {
        if exempt != Some(a) {
            xtx[a * columns + a] += config.penalty;
        }
    }

    let normal = DenseMatrix::from_vec(xtx, columns, columns)
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

    let mut rss = T::zero();
    for (row, yi) in rows {
        let mut fitted = T::zero();
        for (a, &r) in row.iter().enumerate() {
            fitted += beta[a] * r;
        }
        let e = yi - fitted;
        rss += e * e;
    }
    let dof = T::from_usize(if n > columns { n - columns } else { 1 }).ok_or_else(|| {
        StatsError::ConversionFailed(
            "a degrees-of-freedom count is not representable in the working scalar",
        )
    })?;
    Ok(RidgeFit {
        beta,
        sigma2: rss / dof,
    })
}
