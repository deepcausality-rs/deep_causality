/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Logistic regression by iteratively reweighted least squares.

use crate::errors::stats_error::StatsError;
use crate::types::logistic_fit::LogisticFit;
use alloc::vec;
use alloc::vec::Vec;
use deep_causality_algebra::{RealField, Scalar};
use deep_causality_linear::{DenseMatrix, DenseVector, solve};
use deep_causality_num::FromPrimitive;

/// The logistic function `1 / (1 + e^{−x})`.
///
/// Bounded on [`Scalar`] rather than [`deep_causality_algebra::Real`]: the quotient needs
/// division, which `Real` does not carry, but it does not need field invertibility. `Scalar` is
/// exactly that middle, so a dual number flows through here and the derivative comes with it.
pub fn sigmoid<T: Scalar>(x: T) -> T {
    let one = T::one();
    // Two branches so the exponent is never positive. `1/(1 + e^{−x})` overflows for a large
    // negative `x` and `e^x/(1 + e^x)` for a large positive one; taking each where its exponent
    // is negative saturates cleanly at both ends instead.
    if x >= T::zero() {
        one / (one + (-x).exp())
    } else {
        let e = x.exp();
        e / (one + e)
    }
}

/// Fits a ridge-penalised logistic regression by IRLS.
///
/// Reaching `max_iterations` without meeting `tolerance` is a typed error carrying the count, not
/// a returned coefficient vector. An unconverged IRLS iterate on separable data diverges rather
/// than settling, so returning it as a fit would present an arbitrarily large coefficient as an
/// estimate.
pub fn fit_logistic<T>(
    x: &[Vec<T>],
    y: &[T],
    penalty: T,
    max_iterations: usize,
    tolerance: T,
) -> Result<LogisticFit<T>, StatsError>
where
    T: RealField + FromPrimitive,
{
    if x.is_empty() {
        return Err(StatsError::EmptyInput(
            "a logistic fit needs at least one row",
        ));
    }
    if y.len() != x.len() {
        return Err(StatsError::DimensionMismatch(
            "the label vector has a different length from the design",
        ));
    }
    let p = x[0].len();
    if p == 0 || x.iter().any(|r| r.len() != p) {
        return Err(StatsError::DimensionMismatch(
            "every design row must carry the same non-zero number of columns",
        ));
    }
    if x.iter().flatten().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return Err(StatsError::NonFiniteInput(
            "a non-finite observation has no logistic fit",
        ));
    }
    if !penalty.is_finite() || !tolerance.is_finite() {
        return Err(StatsError::NonFiniteInput(
            "a non-finite penalty or tolerance does not define a stopping test",
        ));
    }
    let one = T::one();
    // A logistic label IS a probability — 0, 1, or a proportion between them — so a value
    // outside the unit interval is refused as one, at either end.
    if y.iter().any(|&v| v < T::zero() || v > one) {
        return Err(StatsError::NegativeProbability(
            "a logistic label lies in [0, 1]: it is a probability, and nothing outside that \
             interval is one",
        ));
    }

    let mut beta = vec![T::zero(); p];

    // Newton / IRLS on the ridge-penalised log-likelihood:
    //   gradient  g = Xᵀ(y − p) − λβ
    //   Hessian   H = XᵀWX + λI,   W = diag(pᵢ(1 − pᵢ))
    //   step      Δ = H⁻¹g,  β ← β + Δ
    // The weights vanish as a fitted probability saturates, so `H` loses rank exactly where the
    // likelihood flattens. That is the separable case, and it surfaces here as a vanishing pivot
    // rather than as an arbitrarily large step returned as an estimate.
    for iteration in 1..=max_iterations {
        let mut grad = vec![T::zero(); p];
        let mut hess = vec![T::zero(); p * p];

        for (row, &yi) in x.iter().zip(y.iter()) {
            let mut eta = T::zero();
            for (a, &r) in row.iter().enumerate() {
                eta += beta[a] * r;
            }
            let pi = sigmoid(eta);
            let w = pi * (one - pi);
            let resid = yi - pi;
            for a in 0..p {
                grad[a] += row[a] * resid;
                let ra = row[a];
                for b in 0..p {
                    hess[a * p + b] += ra * w * row[b];
                }
            }
        }
        for a in 0..p {
            grad[a] -= penalty * beta[a];
            hess[a * p + a] += penalty;
        }

        // The Newton step goes through `deep_causality_linear`, as the ridge solve does. The
        // Hessian is symmetric and, for a positive penalty, positive definite — but the same
        // argument against Cholesky applies: a singular Hessian is the signal this loop reads,
        // and it has to arrive as an error rather than as a floored pivot.
        let hessian = DenseMatrix::from_vec(hess, p, p)
            .map_err(|_| StatsError::DimensionMismatch("the Hessian is not square"))?;
        let step = solve(&hessian, &DenseVector::from_vec(grad))
            .map(|v| v.as_slice().to_vec())
            .map_err(|_| {
                // A singular Hessian means two different things, and the iteration number tells them
                // apart. On the first pass `β = 0`, so every weight is `¼` and `H = ¼XᵀX + λI`: a
                // singularity there is a property of the DESIGN, which is rank deficient. Later the
                // weights have moved, and a singularity means they collapsed — the fitted
                // probabilities saturated, which is separation, and the likelihood has no finite
                // maximum to converge to.
                if iteration == 1 {
                    StatsError::RankDeficient(
                        "the design is rank deficient at this penalty: the Hessian is singular before \
                         the first step, where the weights are still uniform",
                    )
                } else {
                    StatsError::NotConverged(
                        iteration,
                        "the fitted probabilities saturated and the weights collapsed: the data are \
                         separable, so the likelihood has no finite maximum and any coefficient \
                         returned here would be an artefact of the stopping point",
                    )
                }
            })?;

        let mut largest = T::zero();
        for a in 0..p {
            beta[a] += step[a];
            let m = step[a].abs();
            if m > largest {
                largest = m;
            }
        }
        if !beta.iter().all(|v| v.is_finite()) {
            return Err(StatsError::NotConverged(
                iteration,
                "the iterate left the representable range: the likelihood has no finite maximum here",
            ));
        }
        if largest <= tolerance {
            return Ok(LogisticFit {
                beta,
                iterations: iteration,
            });
        }
    }

    Err(StatsError::NotConverged(
        max_iterations,
        "the step never fell below the tolerance",
    ))
}
