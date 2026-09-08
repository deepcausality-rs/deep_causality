/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Phase-2 suite for `fit_ridge` and `fit_ridge_streaming`.
//!
//! # Where every expected value comes from
//!
//! The documented model is `β = (XᵀX + λI)⁻¹ Xᵀy`, with `σ² = ‖y − Xβ‖² / max(n − p, 1)`
//! (the degrees-of-freedom convention is stated on [`RidgeFit::sigma2`]). Every expectation
//! below is one of:
//!
//! * a **closed form solved by hand** on integer data, with the arithmetic written out in the
//!   comment above the literal so a reader can redo it without running anything;
//! * an **algebraic invariant** (an exactly-determined system returns its generator; a zero
//!   response returns a zero coefficient vector; a symmetric design returns symmetric
//!   coefficients; a residual of zero has a residual variance of zero);
//! * a **property over a generated family** (the coefficient norm decreases as the penalty
//!   increases; the residual variance is never negative);
//! * an **agreement check** between the materialised and the streaming form, which the spec
//!   requires. That one is never the only authority for either function: the streaming form is
//!   pinned to the same hand-solved closed form on its own, in
//!   `test_fit_ridge_streaming_one_column_closed_form`.
//!
//! Nothing here is taken from the BRCD implementation this crate absorbs, and nothing is a
//! recorded first run.
//!
//! # Corner-case enumeration
//!
//! | Row | Case | Covered by |
//! |-----|------|-----------|
//! | A | Empty input | `test_fit_ridge_rejects_empty_design`, `test_fit_ridge_streaming_rejects_empty_stream` |
//! | B | Single element | `test_fit_ridge_single_observation_single_column` |
//! | C | Two quantities coincide | `test_fit_ridge_identity_design`, `test_fit_ridge_rank_deficient_design` (duplicated column) |
//! | D | Index expression degenerates | `test_fit_ridge_one_column_closed_form` (`p = 1`, so `a*p + b` is always 0), `test_fit_ridge_underdetermined_dof_floor` (`n = 1`), `test_fit_ridge_rejects_zero_width_design` |
//! | E | Documented threshold, both sides | `test_fit_ridge_rank_deficient_design`: the doc's threshold is `λ = 0`; refused at exactly zero, accepted just above it and at `λ = 1` |
//! | F | Zero | `test_fit_ridge_zero_response`, and `λ = 0` throughout |
//! | G | Negative | `test_fit_ridge_negative_penalty_follows_the_sign_convention`, negative generating coefficient in `test_fit_ridge_exactly_determined_recovers_generator` |
//! | H | Exact domain boundary | `test_fit_ridge_exactly_determined_recovers_generator` (`n = p`, where `n − p` is exactly 0), `test_fit_ridge_underdetermined_dof_floor` (`n < p`, where `n − p` is negative) |
//! | I | Non-finite | `test_fit_ridge_non_finite_input_is_not_laundered` |
//! | J | Overflow / underflow reach | `test_fit_ridge_at_the_types_own_extremes`, parameterised per precision |
//! | K | `f32`, `f64`, `Float106` | Every numeric test is a generic helper called three times with its own tolerance |

// `Real` and `ToPrimitive` are supertraits of `RealField`, so the analytic surface (`abs`,
// `sqrt`, `is_finite`, `nan`) and the crossing back to `f64` used in assertion messages both
// resolve through the `RealField` bound without a second import.
use deep_causality_algebra::RealField;
use deep_causality_num::lift;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_stats::utils_tests::lift_array;
use deep_causality_stats::utils_tests::precision::{F32, F64, F106};
use deep_causality_stats::{
    Penalisation, RidgeConfig, RidgeFit, StatsError, StatsErrorEnum, fit_ridge, fit_ridge_streaming,
};

/// Lifts a list of `f64` rows into a design matrix in the working scalar.
fn design<T: FromPrimitive>(rows: &[&[f64]]) -> Vec<Vec<T>> {
    rows.iter().map(|r| lift_array::<T>(r)).collect()
}

/// Pairs a design with its response for the streaming form.
fn stream_rows<T: Copy>(x: &[Vec<T>], y: &[T]) -> Vec<(Vec<T>, T)> {
    x.iter().cloned().zip(y.iter().copied()).collect()
}

/// Euclidean length of a coefficient vector.
fn norm<T: RealField>(beta: &[T]) -> T {
    let mut acc = T::zero();
    for &b in beta {
        acc += b * b;
    }
    acc.sqrt()
}

/// `xᵀβ` for one design row.
fn predict<T: RealField>(beta: &[T], row: &[T]) -> T {
    let mut acc = T::zero();
    for (&b, &v) in beta.iter().zip(row.iter()) {
        acc += b * v;
    }
    acc
}

/// Unwraps a fit without requiring `T: Debug`.
fn fit_of<T>(r: Result<RidgeFit<T>, StatsError>) -> RidgeFit<T> {
    match r {
        Ok(f) => f,
        Err(e) => panic!("expected a fit, got error: {e}"),
    }
}

/// Unwraps an error variant without requiring `T: Debug`.
fn err_of<T>(r: Result<RidgeFit<T>, StatsError>) -> StatsErrorEnum {
    match r {
        Ok(_) => panic!("expected a typed error, got a fit"),
        Err(e) => e.0,
    }
}

/// Asserts `actual ≈ expected` within an absolute tolerance chosen per precision.
fn assert_close<T: RealField + FromPrimitive>(actual: T, expected: f64, tol: f64, what: &str) {
    let want = lift::<T>(expected);
    let diff = (actual - want).abs();
    assert!(
        diff <= lift::<T>(tol),
        "{what}: expected {expected}, got {} (tolerance {tol})",
        actual.to_f64().unwrap_or(f64::NAN)
    );
}

// ---------------------------------------------------------------------------------------------
// 1. Closed form on a one-column design.
// ---------------------------------------------------------------------------------------------

/// Closed form for a single column, derived by hand:
///
/// With one column, `XᵀX = Σxᵢ²` and `Xᵀy = Σxᵢyᵢ` are scalars, so
/// `β = Σ(xᵢyᵢ) / (Σxᵢ² + λ)`.
///
/// Data `x = (1, 2, 3)`, `y = (2, 4, 6)`, `λ = 2`:
///   Σxy = 1·2 + 2·4 + 3·6 = 2 + 8 + 18 = 28
///   Σx² = 1 + 4 + 9 = 14
///   β   = 28 / (14 + 2) = 28/16 = **1.75**  (exact in binary)
///
/// Residuals `yᵢ − βxᵢ` = 2 − 1.75 = 0.25; 4 − 3.5 = 0.5; 6 − 5.25 = 0.75
///   RSS = 0.0625 + 0.25 + 0.5625 = 0.875
///   dof = max(n − p, 1) = max(3 − 1, 1) = 2
///   σ²  = 0.875 / 2 = **0.4375**  (exact in binary)
///
/// This is also the non-vanishing pin for `sigma2`: several other tests assert it is zero, and a
/// quantity asserted only where it vanishes would survive a wrong divisor or a dropped square.
///
/// Corner row D: with `p = 1` every index into a `p × p` normal-equation buffer collapses to 0.
fn check_one_column_closed_form<T: RealField + FromPrimitive>(tol: f64) {
    let x = design::<T>(&[&[1.0], &[2.0], &[3.0]]);
    let y = lift_array::<T>(&[2.0, 4.0, 6.0]);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(2.0))));

    assert_eq!(fit.beta.len(), 1, "one column in, one coefficient out");
    assert_close(fit.beta[0], 1.75, tol, "beta = 28/16");
    assert_close(fit.sigma2, 0.4375, tol, "sigma2 = 0.875/2");
}

#[test]
fn test_fit_ridge_one_column_closed_form() {
    check_one_column_closed_form::<f32>(F32.solve);
    check_one_column_closed_form::<f64>(F64.solve);
    check_one_column_closed_form::<Float106>(F106.solve);
}

// ---------------------------------------------------------------------------------------------
// 2. Single observation (corner row B).
// ---------------------------------------------------------------------------------------------

/// One row, one column, from the same hand-derived scalar closed form
/// `β = Σ(xᵢyᵢ) / (Σxᵢ² + λ)`.
///
/// Data `x = (2)`, `y = (6)`:
///   Σxy = 12, Σx² = 4
///   λ = 0: β = 12 / 4 = **3**; the fit is exact, so RSS = 0 and σ² = 0
///   λ = 2: β = 12 / 6 = **2**; residual = 6 − 2·2 = 2, RSS = 4,
///          dof = max(1 − 1, 1) = 1, so σ² = 4/1 = **4**
///
/// The `λ = 2` branch is the second non-vanishing `sigma2` pin, and it pins the dof floor: with
/// `n = p = 1` the divisor is the floor 1, not `n − p = 0`.
fn check_single_observation<T: RealField + FromPrimitive>(tol: f64, zero_tol: f64) {
    let x = design::<T>(&[&[2.0]]);
    let y = lift_array::<T>(&[6.0]);

    let unpenalised = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::zero())));
    assert_close(unpenalised.beta[0], 3.0, tol, "beta = 12/4");
    assert_close(
        unpenalised.sigma2,
        0.0,
        zero_tol,
        "an exact fit has no residual",
    );

    let penalised = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(2.0))));
    assert_close(penalised.beta[0], 2.0, tol, "beta = 12/6");
    assert_close(penalised.sigma2, 4.0, tol, "sigma2 = 4/1 on the dof floor");
}

#[test]
fn test_fit_ridge_single_observation_single_column() {
    check_single_observation::<f32>(F32.solve, F32.zero);
    check_single_observation::<f64>(F64.solve, F64.zero);
    check_single_observation::<Float106>(F106.solve, F106.zero);
}

// ---------------------------------------------------------------------------------------------
// 3. An exactly-determined system at penalty 0 returns its generator.
// ---------------------------------------------------------------------------------------------

/// Invariant: at `λ = 0` an invertible square design returns exactly the coefficients that
/// generated the response, because `β = X⁻¹y` and `y = Xβ_true` by construction.
///
/// `X = [[1, 1], [1, 2]]`, `det X = 1·2 − 1·1 = 1`, so `X⁻¹ = [[2, −1], [−1, 1]]`.
/// Generating `β_true = (3, −4)` gives
///   y₁ = 1·3 + 1·(−4) = −1
///   y₂ = 1·3 + 2·(−4) = −5
/// and back:
///   β₁ = 2·(−1) − 1·(−5) = −2 + 5 = **3**
///   β₂ = −1·(−1) + 1·(−5) = 1 − 5 = **−4**
///
/// Corner row G: the second coefficient is negative, so a solver that lost a sign would show it
/// here rather than passing on all-positive data. Corner row H: `n = p = 2`, so `n − p` is
/// exactly 0 and the documented `max(n − p, 1)` floor is the divisor.
fn check_exactly_determined<T: RealField + FromPrimitive>(tol: f64, zero_tol: f64) {
    let x = design::<T>(&[&[1.0, 1.0], &[1.0, 2.0]]);
    let y = lift_array::<T>(&[-1.0, -5.0]);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::zero())));

    assert_eq!(fit.beta.len(), 2);
    assert_close(fit.beta[0], 3.0, tol, "generator beta_1");
    assert_close(fit.beta[1], -4.0, tol, "generator beta_2");

    // A square invertible design interpolates its response, so the residual is zero by
    // construction, not by the formula under test.
    for (row, &yi) in x.iter().zip(y.iter()) {
        let r = yi - predict(&fit.beta, row);
        assert!(
            r.abs() <= lift::<T>(zero_tol),
            "an exactly-determined fit interpolates every row"
        );
    }
    assert_close(fit.sigma2, 0.0, zero_tol, "zero residual, zero variance");
}

#[test]
fn test_fit_ridge_exactly_determined_recovers_generator() {
    check_exactly_determined::<f32>(F32.solve, F32.zero);
    check_exactly_determined::<f64>(F64.solve, F64.zero);
    check_exactly_determined::<Float106>(F106.solve, F106.zero);
}

// ---------------------------------------------------------------------------------------------
// 4. Identity design (corner row C: two distinct quantities coincide).
// ---------------------------------------------------------------------------------------------

/// The identity design is the case where several different formulas agree, so it is worth
/// pinning both sides of the penalty.
///
/// With `X = I₂`: `XᵀX = I`, `Xᵀy = y`, so `(I + λI)β = y` and `β = y / (1 + λ)` componentwise.
///
/// `y = (5, 7)`:
///   λ = 0 → β = (5, 7); the design interpolates, so RSS = 0 and σ² = 0
///   λ = 1 → β = (5/2, 7/2) = (**2.5**, **3.5**), both exact in binary
///           predictions are (2.5, 3.5), residuals (5 − 2.5, 7 − 3.5) = (2.5, 3.5)
///           RSS = 6.25 + 12.25 = 18.5, dof = max(2 − 2, 1) = 1, σ² = **18.5**
fn check_identity_design<T: RealField + FromPrimitive>(tol: f64, zero_tol: f64) {
    let x = design::<T>(&[&[1.0, 0.0], &[0.0, 1.0]]);
    let y = lift_array::<T>(&[5.0, 7.0]);

    let unpenalised = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::zero())));
    assert_close(unpenalised.beta[0], 5.0, tol, "identity design returns y");
    assert_close(unpenalised.beta[1], 7.0, tol, "identity design returns y");
    assert_close(
        unpenalised.sigma2,
        0.0,
        zero_tol,
        "interpolation, no residual",
    );

    let penalised = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::one())));
    assert_close(penalised.beta[0], 2.5, tol, "5/(1+1)");
    assert_close(penalised.beta[1], 3.5, tol, "7/(1+1)");
    assert_close(penalised.sigma2, 18.5, tol, "18.5/1");
}

#[test]
fn test_fit_ridge_identity_design() {
    check_identity_design::<f32>(F32.solve, F32.zero);
    check_identity_design::<f64>(F64.solve, F64.zero);
    check_identity_design::<Float106>(F106.solve, F106.zero);
}

// ---------------------------------------------------------------------------------------------
// 5. Two columns, closed form solved by hand, penalty on every column.
// ---------------------------------------------------------------------------------------------

/// The four-row design used by this test and by the shrinkage property below.
///
/// `X = [[1,1], [1,2], [1,3], [1,4]]`, `y = (1, 3, 2, 5)`. Column one is an intercept column.
///
/// Hand-computed normal-equation pieces (used by every derivation in this file that names this
/// design; they are sums over the data, not the formula under test):
///   XᵀX = [[4, 10], [10, 30]]   since Σ1 = 4, Σx = 1+2+3+4 = 10, Σx² = 1+4+9+16 = 30
///   Xᵀy = (11, 33)              since Σy = 1+3+2+5 = 11, Σxy = 1+6+6+20 = 33
fn shrinkage_design<T: FromPrimitive>() -> (Vec<Vec<T>>, Vec<T>) {
    let x = design::<T>(&[&[1.0, 1.0], &[1.0, 2.0], &[1.0, 3.0], &[1.0, 4.0]]);
    let y = lift_array::<T>(&[1.0, 3.0, 2.0, 5.0]);
    (x, y)
}

/// Solved by hand from `(XᵀX + λI)β = Xᵀy` with the pieces above, by Cramer's rule.
///
/// λ = 0: matrix [[4, 10], [10, 30]], det = 120 − 100 = 20
///   β₁ = (11·30 − 10·33)/20 = (330 − 330)/20 = **0**
///   β₂ = (4·33 − 10·11)/20  = (132 − 110)/20 = 22/20 = **1.1**
///
/// λ = 1: matrix [[5, 10], [10, 31]], det = 155 − 100 = 55
///   β₁ = (11·31 − 10·33)/55 = (341 − 330)/55 = 11/55 = **0.2**
///   β₂ = (5·33 − 10·11)/55  = (165 − 110)/55 = 55/55 = **1.0**
///
/// The intercept coefficient moves from 0 to 0.2 as the penalty is switched on, which is what
/// pins the documented `λI` — a penalty applied to *every* column, the intercept included. An
/// implementation that exempted column one would leave β₁ at 0.
///
/// 11/55 = 0.2 and 22/20 = 1.1 are not exact in binary, so these use the `DECIMAL_*` budget.
fn check_penalty_hits_every_column<T: RealField + FromPrimitive>(tol: f64) {
    let (x, y) = shrinkage_design::<T>();

    let unpenalised = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::zero())));
    assert_close(unpenalised.beta[0], 0.0, tol, "intercept at lambda 0");
    assert_close(unpenalised.beta[1], 1.1, tol, "slope 22/20 at lambda 0");

    let penalised = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::one())));
    assert_close(penalised.beta[0], 0.2, tol, "intercept 11/55 at lambda 1");
    assert_close(penalised.beta[1], 1.0, tol, "slope 55/55 at lambda 1");
}

#[test]
fn test_fit_ridge_penalty_applies_to_every_column() {
    check_penalty_hits_every_column::<f32>(F32.literal);
    check_penalty_hits_every_column::<f64>(F64.literal);
    check_penalty_hits_every_column::<Float106>(F106.literal);
}

// ---------------------------------------------------------------------------------------------
// 6. Monotone shrinkage: a property over a generated family.
// ---------------------------------------------------------------------------------------------

/// The coefficient norm strictly decreases as the penalty increases.
///
/// Published result, not a retyped formula: Hoerl, A. E. and Kennard, R. W. (1970), "Ridge
/// Regression: Biased Estimation for Nonorthogonal Problems", *Technometrics* 12(1), 55–67 —
/// Theorem 4.2 states that `‖β̂(k)‖` is a monotone decreasing function of `k`.
///
/// The penalties are spread over four decades so that consecutive norms differ by far more than
/// any precision's rounding noise. From the hand-solved values above the first gap alone is
/// `1.1 − √(0.2² + 1²) = 1.1 − 1.0198 ≈ 0.08`, and the later gaps are larger still; the margin
/// asserted below is orders of magnitude smaller than that, so the test measures the property
/// and not the arithmetic.
fn check_monotone_shrinkage<T: RealField + FromPrimitive>(margin: f64) {
    let (x, y) = shrinkage_design::<T>();
    let penalties = [0.0, 1.0, 10.0, 100.0, 1000.0];

    let norms: Vec<T> = penalties
        .iter()
        .map(|&lambda| {
            let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(lambda))));
            norm(&fit.beta)
        })
        .collect();

    for w in norms.windows(2) {
        assert!(
            w[1] < w[0] - lift::<T>(margin),
            "a larger penalty must give a strictly smaller coefficient norm: {} then {}",
            w[0].to_f64().unwrap_or(f64::NAN),
            w[1].to_f64().unwrap_or(f64::NAN)
        );
    }

    // A norm is non-negative; the largest penalty has not driven it below zero.
    assert!(norms[norms.len() - 1] >= T::zero());
}

#[test]
fn test_fit_ridge_coefficient_norm_shrinks_monotonically() {
    check_monotone_shrinkage::<f32>(F32.solve);
    check_monotone_shrinkage::<f64>(F64.solve);
    check_monotone_shrinkage::<Float106>(F106.solve);
}

// ---------------------------------------------------------------------------------------------
// 7. Fewer rows than columns: the degrees-of-freedom floor (corner row H).
// ---------------------------------------------------------------------------------------------

/// One row, two columns, at `λ = 1`. Hand-solved from `(XᵀX + λI)β = Xᵀy`.
///
/// `X = [[1, 2]]`, `y = (3)`:
///   XᵀX = [[1, 2], [2, 4]], so XᵀX + I = [[2, 2], [2, 5]]
///   Xᵀy = (3, 6)
///   2β₁ + 2β₂ = 3
///   2β₁ + 5β₂ = 6   → subtract: 3β₂ = 3 → β₂ = **1.0**, then 2β₁ = 3 − 2 = 1 → β₁ = **0.5**
///
///   prediction = 0.5·1 + 1·2 = 2.5, residual = 3 − 2.5 = 0.5, RSS = **0.25**
///   dof = max(n − p, 1) = max(1 − 2, 1) = 1, so σ² = 0.25 / 1 = **0.25**
///
/// Both values are exact in binary. This is the case where `n − p` is *negative*: the floor is
/// what stops the divisor being zero or a negative count, and the value 0.25 distinguishes a
/// divisor of 1 from any other choice.
fn check_underdetermined_dof_floor<T: RealField + FromPrimitive>(tol: f64) {
    let x = design::<T>(&[&[1.0, 2.0]]);
    let y = lift_array::<T>(&[3.0]);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::one())));

    assert_eq!(fit.beta.len(), 2);
    assert_close(fit.beta[0], 0.5, tol, "beta_1 from the 2x2 solve");
    assert_close(fit.beta[1], 1.0, tol, "beta_2 from the 2x2 solve");
    assert_close(fit.sigma2, 0.25, tol, "RSS 0.25 over the dof floor of 1");
}

#[test]
fn test_fit_ridge_underdetermined_dof_floor() {
    check_underdetermined_dof_floor::<f32>(F32.solve);
    check_underdetermined_dof_floor::<f64>(F64.solve);
    check_underdetermined_dof_floor::<Float106>(F106.solve);
}

// ---------------------------------------------------------------------------------------------
// 8. A perfectly-fitting over-determined design.
// ---------------------------------------------------------------------------------------------

/// Invariant: when the response lies exactly in the column space, the residual is zero at every
/// row and therefore the residual variance is zero — with more rows than columns, so the fit is
/// not interpolating by construction as it is in test 3.
///
/// `X = [[1,1], [1,2], [1,3]]` with `β_true = (1, 2)` gives
///   y = (1 + 2·1, 1 + 2·2, 1 + 2·3) = (3, 5, 7)
/// At `λ = 0` least squares returns `β_true` because the residual it minimises can be driven to
/// exactly zero and the design has full column rank (its two columns are not proportional).
fn check_perfect_fit<T: RealField + FromPrimitive>(tol: f64, zero_tol: f64) {
    let x = design::<T>(&[&[1.0, 1.0], &[1.0, 2.0], &[1.0, 3.0]]);
    let y = lift_array::<T>(&[3.0, 5.0, 7.0]);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::zero())));

    assert_close(fit.beta[0], 1.0, tol, "intercept");
    assert_close(fit.beta[1], 2.0, tol, "slope");

    for (row, &yi) in x.iter().zip(y.iter()) {
        let r = yi - predict(&fit.beta, row);
        assert!(
            r.abs() <= lift::<T>(zero_tol),
            "a perfectly-fitting design leaves no residual"
        );
    }
    assert!(fit.sigma2 >= T::zero(), "a variance is never negative");
    assert_close(fit.sigma2, 0.0, zero_tol, "zero residual, zero variance");
}

#[test]
fn test_fit_ridge_perfect_fit_has_zero_residual() {
    check_perfect_fit::<f32>(F32.solve, F32.zero);
    check_perfect_fit::<f64>(F64.solve, F64.zero);
    check_perfect_fit::<Float106>(F106.solve, F106.zero);
}

// ---------------------------------------------------------------------------------------------
// 9. Zero response (corner row F).
// ---------------------------------------------------------------------------------------------

/// Invariant: `Xᵀy = 0` when `y = 0`, so the normal equations read `(XᵀX + λI)β = 0` and the
/// unique solution of a non-singular system is `β = 0`. Every prediction is then zero, so the
/// residual and the residual variance are zero too.
///
/// Checked at a positive penalty, where the system is non-singular whatever the design.
fn check_zero_response<T: RealField + FromPrimitive>(zero_tol: f64) {
    let x = design::<T>(&[&[1.0, 1.0], &[1.0, 2.0], &[1.0, 3.0]]);
    let y = lift_array::<T>(&[0.0, 0.0, 0.0]);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::one())));

    for &b in &fit.beta {
        assert!(
            b.abs() <= lift::<T>(zero_tol),
            "a zero response has zero coefficients"
        );
    }
    assert!(fit.sigma2 >= T::zero());
    assert_close(fit.sigma2, 0.0, zero_tol, "no response, no residual");
}

#[test]
fn test_fit_ridge_zero_response() {
    check_zero_response::<f32>(F32.zero);
    check_zero_response::<f64>(F64.zero);
    check_zero_response::<Float106>(F106.zero);
}

// ---------------------------------------------------------------------------------------------
// 10. Rank deficiency and the documented `λ = 0` threshold (corner rows C and E).
// ---------------------------------------------------------------------------------------------

/// A duplicated column: `X = [[1,1], [2,2], [3,3]]`. The two columns are identical, so `XᵀX` is
/// singular and at `λ = 0` no unique `β` exists — the docstring says this is refused, and the
/// error's own documentation ("no unique solution at the penalty given") names the variant.
///
/// Above the threshold the system is non-singular and the solve succeeds. At `λ = 1`, by hand:
///   Σx² over either column = 1 + 4 + 9 = 14, and the cross term is the same 14
///   XᵀX + I = [[15, 14], [14, 15]]
///   Xᵀy with y = (1, 2, 3) = (1 + 4 + 9, 1 + 4 + 9) = (14, 14)
///   The system is symmetric in the two coefficients, so β₁ = β₂ = b with
///   (15 + 14)·b = 14 → b = 14/29 = **0.4827586206896551…**
///
/// 14/29 is not exact in binary, so this uses the `DECIMAL_*` budget.
///
/// Just above the threshold (`λ = 1e-3`) the test asserts only that the solve succeeds and that
/// the two coefficients are equal — the design's own symmetry, which no rounding can break in
/// principle — because a decimal for `14/28.001` would be pinning the conditioning rather than
/// the mathematics.
fn check_rank_deficient<T: RealField + FromPrimitive>(tol: f64, symmetry_tol: f64) {
    let x = design::<T>(&[&[1.0, 1.0], &[2.0, 2.0], &[3.0, 3.0]]);
    let y = lift_array::<T>(&[1.0, 2.0, 3.0]);

    // At exactly the documented threshold: refused.
    let err = err_of(fit_ridge(&x, &y, &RidgeConfig::new(T::zero())));
    assert!(
        matches!(err, StatsErrorEnum::RankDeficient(_)),
        "a duplicated column at zero penalty has no unique solution"
    );

    // Just above it: accepted, and symmetric in the two identical columns.
    let just_above = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(1e-3))));
    assert!(
        (just_above.beta[0] - just_above.beta[1]).abs() <= lift::<T>(symmetry_tol),
        "identical columns must receive identical coefficients"
    );

    // Well above it: the hand-solved value.
    let penalised = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::one())));
    assert_close(penalised.beta[0], 0.4827586206896552, tol, "14/29");
    assert_close(penalised.beta[1], 0.4827586206896552, tol, "14/29");
    assert!(
        penalised.sigma2 >= T::zero(),
        "a variance is never negative"
    );
}

#[test]
fn test_fit_ridge_rank_deficient_design() {
    // The symmetry tolerance is looser than the value tolerance at every precision: at
    // lambda = 1e-3 the two-column system is deliberately near-singular, so the two
    // coefficients agree to fewer digits than a well-conditioned solve would give.
    check_rank_deficient::<f32>(F32.literal, 1e-3);
    check_rank_deficient::<f64>(F64.literal, 1e-10);
    check_rank_deficient::<Float106>(F106.literal, 1e-10);
}

// ---------------------------------------------------------------------------------------------
// 11. Negative penalty (corner row G), including the one that cancels the design.
// ---------------------------------------------------------------------------------------------

/// The penalty enters as `XᵀX + λI`, so a negative `λ` subtracts. Both branches come from the
/// same hand-derived one-column closed form `β = Σ(xᵢyᵢ) / (Σxᵢ² + λ)`.
///
/// Data `x = (2)`, `y = (6)`, so Σxy = 12 and Σx² = 4:
///   λ = −2: β = 12 / (4 − 2) = 12/2 = **6**; prediction 12, residual 6 − 12 = −6,
///           RSS = 36, dof = max(1 − 1, 1) = 1, σ² = **36**
///   λ = −4: 4 + (−4) = 0 — the penalty cancels the design's own curvature exactly, so the
///           system is singular and `RankDeficient` ("no unique solution at the penalty given")
///           is the documented answer.
///
/// The `λ = −2` branch is what distinguishes `XᵀX + λI` from `XᵀX + |λ|I`: with the absolute
/// value the answer would be 2, not 6.
fn check_negative_penalty<T: RealField + FromPrimitive>(tol: f64) {
    let x = design::<T>(&[&[2.0]]);
    let y = lift_array::<T>(&[6.0]);

    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(-2.0))));
    assert_close(fit.beta[0], 6.0, tol, "12/(4-2)");
    assert_close(fit.sigma2, 36.0, tol, "RSS 36 over the dof floor of 1");

    let err = err_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(-4.0))));
    assert!(
        matches!(err, StatsErrorEnum::RankDeficient(_)),
        "a penalty that cancels the design leaves no unique solution"
    );
}

#[test]
fn test_fit_ridge_negative_penalty_follows_the_sign_convention() {
    check_negative_penalty::<f32>(F32.solve);
    check_negative_penalty::<f64>(F64.solve);
    check_negative_penalty::<Float106>(F106.solve);
}

// ---------------------------------------------------------------------------------------------
// 12. `sigma2` is never negative: a property over a generated family.
// ---------------------------------------------------------------------------------------------

/// `σ²` is a sum of squares over a positive divisor, so it is non-negative for every design and
/// every non-negative penalty. Asserted over a family rather than at one point.
fn check_sigma2_non_negative<T: RealField + FromPrimitive>() {
    let designs: [(&[&[f64]], &[f64]); 3] = [
        (&[&[1.0, 1.0], &[1.0, 2.0], &[1.0, 3.0]], &[1.0, 4.0, 2.0]),
        (&[&[2.0], &[-3.0], &[5.0]], &[-1.0, 0.5, 2.0]),
        (
            &[&[1.0, 0.0, 2.0], &[0.0, 1.0, -1.0], &[3.0, 1.0, 0.0]],
            &[1.0, -2.0, 4.0],
        ),
    ];

    for (rows, response) in designs {
        let x = design::<T>(rows);
        let y = lift_array::<T>(response);
        for lambda in [0.5, 1.0, 4.0, 32.0] {
            let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(lambda))));
            assert!(
                fit.sigma2 >= T::zero(),
                "a residual variance is a sum of squares over a positive divisor"
            );
            assert!(
                fit.sigma2.is_finite(),
                "finite data gives a finite variance"
            );
        }
    }
}

#[test]
fn test_fit_ridge_sigma2_is_never_negative() {
    check_sigma2_non_negative::<f32>();
    check_sigma2_non_negative::<f64>();
    check_sigma2_non_negative::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// 13-16. Typed errors on malformed input (corner rows A and D).
// ---------------------------------------------------------------------------------------------

/// No observations at all. `StatsErrorEnum::EmptyInput` is documented as "a statistic was asked
/// for over no observations", which is exactly this.
fn check_empty_design<T: RealField + FromPrimitive>() {
    let x: Vec<Vec<T>> = Vec::new();
    let y: Vec<T> = Vec::new();
    let err = err_of(fit_ridge(&x, &y, &RidgeConfig::new(T::one())));
    assert!(matches!(err, StatsErrorEnum::EmptyInput(_)));
}

#[test]
fn test_fit_ridge_rejects_empty_design() {
    check_empty_design::<f32>();
    check_empty_design::<f64>();
    check_empty_design::<Float106>();
}

/// Three design rows and two responses. `DimensionMismatch` is documented as "two inputs that
/// must agree on length or shape do not".
fn check_response_length_mismatch<T: RealField + FromPrimitive>() {
    let x = design::<T>(&[&[1.0], &[2.0], &[3.0]]);
    let y = lift_array::<T>(&[1.0, 2.0]);
    let err = err_of(fit_ridge(&x, &y, &RidgeConfig::new(T::one())));
    assert!(matches!(err, StatsErrorEnum::DimensionMismatch(_)));

    // And the other direction: more responses than rows.
    let x_short = design::<T>(&[&[1.0], &[2.0]]);
    let y_long = lift_array::<T>(&[1.0, 2.0, 3.0]);
    let err = err_of(fit_ridge(&x_short, &y_long, &RidgeConfig::new(T::one())));
    assert!(matches!(err, StatsErrorEnum::DimensionMismatch(_)));
}

#[test]
fn test_fit_ridge_rejects_response_length_mismatch() {
    check_response_length_mismatch::<f32>();
    check_response_length_mismatch::<f64>();
    check_response_length_mismatch::<Float106>();
}

/// Rows of differing width. The second row is short, so the design is not a matrix.
fn check_ragged_rows<T: RealField + FromPrimitive>() {
    let x = design::<T>(&[&[1.0, 2.0], &[3.0], &[4.0, 5.0]]);
    let y = lift_array::<T>(&[1.0, 2.0, 3.0]);
    let err = err_of(fit_ridge(&x, &y, &RidgeConfig::new(T::one())));
    assert!(matches!(err, StatsErrorEnum::DimensionMismatch(_)));

    // A long row is the same defect in the other direction.
    let x_long = design::<T>(&[&[1.0, 2.0], &[3.0, 4.0, 5.0]]);
    let y2 = lift_array::<T>(&[1.0, 2.0]);
    let err = err_of(fit_ridge(&x_long, &y2, &RidgeConfig::new(T::one())));
    assert!(matches!(err, StatsErrorEnum::DimensionMismatch(_)));
}

#[test]
fn test_fit_ridge_rejects_ragged_rows() {
    check_ragged_rows::<f32>();
    check_ragged_rows::<f64>();
    check_ragged_rows::<Float106>();
}

/// Rows that exist but carry no columns: `p = 0`, so there is nothing to estimate and every
/// index into a `p × p` buffer is out of range (corner row D, the collapsed stride).
///
/// Ambiguity, and the reading asserted: the docstrings do not say which variant a zero-width
/// design gets. It is defensible as "no observations to speak of" (`EmptyInput`) and as "a shape
/// the caller got wrong" (`DimensionMismatch`), so the test accepts either and rejects a success
/// or any third variant. The point being pinned is that a zero-width design is refused with a
/// typed error rather than returning an empty `beta` as though it were an answer.
fn check_zero_width_design<T: RealField + FromPrimitive>() {
    let x: Vec<Vec<T>> = vec![Vec::new(), Vec::new(), Vec::new()];
    let y = lift_array::<T>(&[1.0, 2.0, 3.0]);
    let err = err_of(fit_ridge(&x, &y, &RidgeConfig::new(T::one())));
    assert!(matches!(
        err,
        StatsErrorEnum::EmptyInput(_) | StatsErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_fit_ridge_rejects_zero_width_design() {
    check_zero_width_design::<f32>();
    check_zero_width_design::<f64>();
    check_zero_width_design::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// 17. Non-finite input (corner row I).
// ---------------------------------------------------------------------------------------------

/// A `NaN` or an infinity anywhere in the design or the response must not come back as a
/// finite-looking answer.
///
/// Ambiguity, and the reading asserted: `fit_ridge`'s docstring does not say whether it screens
/// its input, and `StatsErrorEnum::NonFiniteInput` exists but is not named for this function.
/// The test therefore accepts either outcome — a typed error, or a fit whose coefficients or
/// variance are themselves non-finite — and rejects only the third possibility, a fit that
/// reports finite numbers computed from data that had none. That is the outcome a caller cannot
/// detect, and the spec's "rather than returning a silently wrong solution" rules it out.
fn check_non_finite_not_laundered<T: RealField + FromPrimitive>() {
    let poisoned: [T; 3] = [
        T::nan(),
        lift::<T>(f64::INFINITY),
        lift::<T>(f64::NEG_INFINITY),
    ];

    for bad in poisoned {
        // In the response.
        let x = design::<T>(&[&[1.0, 1.0], &[1.0, 2.0], &[1.0, 3.0]]);
        let mut y = lift_array::<T>(&[1.0, 2.0, 3.0]);
        y[1] = bad;
        assert_not_laundered(fit_ridge(&x, &y, &RidgeConfig::new(T::one())));

        // In the design.
        let mut x2 = design::<T>(&[&[1.0, 1.0], &[1.0, 2.0], &[1.0, 3.0]]);
        x2[2][1] = bad;
        let y2 = lift_array::<T>(&[1.0, 2.0, 3.0]);
        assert_not_laundered(fit_ridge(&x2, &y2, &RidgeConfig::new(T::one())));

        // In the penalty.
        let x3 = design::<T>(&[&[1.0, 1.0], &[1.0, 2.0], &[1.0, 3.0]]);
        let y3 = lift_array::<T>(&[1.0, 2.0, 3.0]);
        assert_not_laundered(fit_ridge(&x3, &y3, &RidgeConfig::new(bad)));
    }
}

fn assert_not_laundered<T: RealField>(r: Result<RidgeFit<T>, StatsError>) {
    match r {
        Err(_) => {}
        Ok(fit) => assert!(
            fit.beta.iter().any(|b| !b.is_finite()) || !fit.sigma2.is_finite(),
            "non-finite input must not produce a finite-looking fit"
        ),
    }
}

#[test]
fn test_fit_ridge_non_finite_input_is_not_laundered() {
    check_non_finite_not_laundered::<f32>();
    check_non_finite_not_laundered::<f64>();
    check_non_finite_not_laundered::<Float106>();
}

// ---------------------------------------------------------------------------------------------
// 18. The type's own extremes (corner row J).
// ---------------------------------------------------------------------------------------------

/// Overflow and underflow reach, at each type's own extremes rather than at a shared constant.
///
/// The design is one column `x = (s)` with response `y = (s)`. From the hand-derived one-column
/// closed form `β = Σ(xᵢyᵢ) / (Σxᵢ² + λ)`, at `λ = 0` this is `s·s / (s·s) = **1**` for every
/// non-zero `s`, exactly — the answer is scale-free even though the intermediate `s²` is not.
///
/// * `safe` is chosen so `s²` is representable: the fit must return 1.
/// * `over` is chosen so `s` is representable but `s²` is not: the accumulated `XᵀX` overflows.
/// * `under` is chosen so `s` is representable but `s²` underflows to zero: the accumulated
///   `XᵀX` collapses and the system becomes singular.
///
/// For the last two the assertion is the same as for row I: a typed error, or a non-finite
/// result, but never a finite number that is not 1 — the true answer is 1, so a finite 0 or a
/// finite anything-else is the silently wrong solution the spec forbids.
fn check_extremes<T: RealField + FromPrimitive>(safe: f64, over: f64, under: f64, tol: f64) {
    let x = design::<T>(&[&[safe]]);
    let y = lift_array::<T>(&[safe]);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::zero())));
    assert_close(
        fit.beta[0],
        1.0,
        tol,
        "s*s/(s*s) = 1 at the representable extreme",
    );

    for scale in [over, under] {
        let x = design::<T>(&[&[scale]]);
        let y = lift_array::<T>(&[scale]);
        match fit_ridge(&x, &y, &RidgeConfig::new(T::zero())) {
            Err(_) => {}
            Ok(fit) => assert!(
                !fit.beta[0].is_finite() || (fit.beta[0] - T::one()).abs() <= lift::<T>(tol),
                "an unrepresentable intermediate must not yield a finite wrong coefficient"
            ),
        }
    }
}

#[test]
fn test_fit_ridge_at_the_types_own_extremes() {
    // f32: max ~3.4e38, smallest subnormal ~1.4e-45.
    //   1e18^2 = 1e36 fits; 1e20^2 = 1e40 overflows; 1e-25^2 = 1e-50 underflows to zero.
    check_extremes::<f32>(1e18, 1e20, 1e-25, F32.solve);
    // f64: max ~1.8e308, smallest subnormal ~4.9e-324.
    //   1e150^2 = 1e300 fits; 1e180^2 = 1e360 overflows; 1e-180^2 = 1e-360 underflows to zero.
    check_extremes::<f64>(1e150, 1e180, 1e-180, F64.solve);
    // Float106 is a double-double: it carries roughly twice the significand of f64 over the same
    // exponent range, so the reach is f64's and only the precision differs.
    check_extremes::<Float106>(1e150, 1e180, 1e-180, F106.solve);
}

// ---------------------------------------------------------------------------------------------
// 19. The streaming form against the same hand-solved closed form.
// ---------------------------------------------------------------------------------------------

/// The streaming form is pinned to the mathematics on its own, not only to the materialised
/// form. Same data and same derivation as `check_one_column_closed_form`:
///   x = (1, 2, 3), y = (2, 4, 6), λ = 2
///   Σxy = 28, Σx² = 14, β = 28/16 = **1.75**
///   residuals 0.25, 0.5, 0.75 → RSS = 0.875, dof = max(3 − 1, 1) = 2, σ² = **0.4375**
fn check_streaming_closed_form<T: RealField + FromPrimitive>(tol: f64) {
    let x = design::<T>(&[&[1.0], &[2.0], &[3.0]]);
    let y = lift_array::<T>(&[2.0, 4.0, 6.0]);
    let fit = fit_of(fit_ridge_streaming(
        stream_rows(&x, &y),
        &RidgeConfig::new(lift::<T>(2.0)),
        1,
    ));

    assert_eq!(fit.beta.len(), 1);
    assert_close(fit.beta[0], 1.75, tol, "beta = 28/16, streamed");
    assert_close(fit.sigma2, 0.4375, tol, "sigma2 = 0.875/2, streamed");
}

#[test]
fn test_fit_ridge_streaming_one_column_closed_form() {
    check_streaming_closed_form::<f32>(F32.solve);
    check_streaming_closed_form::<f64>(F64.solve);
    check_streaming_closed_form::<Float106>(F106.solve);
}

// ---------------------------------------------------------------------------------------------
// 20-21. The two forms agree.
// ---------------------------------------------------------------------------------------------

/// The docstring's promise: "over the same rows the result agrees with `fit_ridge` to the
/// precision in use".
///
/// This is an agreement test, and it is never the sole authority for either function — each is
/// pinned to a hand-solved closed form elsewhere in this file. What it adds is the question the
/// closed forms cannot answer: whether the accumulate-as-you-go path and the materialised path
/// have drifted apart.
fn check_forms_agree<T: RealField + FromPrimitive>(tol: f64) {
    let x = design::<T>(&[
        &[1.0, 0.5, -2.0],
        &[1.0, 1.5, 0.0],
        &[1.0, -0.5, 3.0],
        &[1.0, 2.5, 1.0],
        &[1.0, 0.0, -1.0],
    ]);
    let y = lift_array::<T>(&[1.0, 2.5, -0.5, 4.0, 0.25]);

    for lambda in [0.0, 0.25, 1.0, 8.0] {
        let penalty = lift::<T>(lambda);
        let dense = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(penalty)));
        let streamed = fit_of(fit_ridge_streaming(
            stream_rows(&x, &y),
            &RidgeConfig::new(penalty),
            3,
        ));

        assert_eq!(dense.beta.len(), streamed.beta.len());
        for (a, b) in dense.beta.iter().zip(streamed.beta.iter()) {
            assert!(
                (*a - *b).abs() <= lift::<T>(tol),
                "the two forms must agree on every coefficient"
            );
        }
        assert!(
            (dense.sigma2 - streamed.sigma2).abs() <= lift::<T>(tol),
            "the two forms must agree on the residual variance"
        );
    }
}

#[test]
fn test_fit_ridge_streaming_agrees_with_materialised() {
    check_forms_agree::<f32>(F32.solve);
    check_forms_agree::<f64>(F64.solve);
    check_forms_agree::<Float106>(F106.solve);
}

/// The same agreement over a *filtered* design, which is the case the streaming form exists for:
/// the caller drops rows as it goes rather than materialising the kept ones. The materialised
/// side is given exactly the rows the filter keeps.
fn check_forms_agree_when_filtered<T: RealField + FromPrimitive>(tol: f64) {
    let all = design::<T>(&[
        &[1.0, 1.0],
        &[1.0, 9.0],
        &[1.0, 2.0],
        &[1.0, 9.0],
        &[1.0, 3.0],
        &[1.0, 4.0],
    ]);
    let response = lift_array::<T>(&[2.0, 100.0, 3.0, 100.0, 5.0, 6.0]);
    let sentinel = lift::<T>(100.0);

    let kept: Vec<Vec<T>> = all
        .iter()
        .zip(response.iter())
        .filter(|&(_, &yi)| yi != sentinel)
        .map(|(row, _)| row.clone())
        .collect();
    let kept_y: Vec<T> = response
        .iter()
        .copied()
        .filter(|&yi| yi != sentinel)
        .collect();

    let penalty = lift::<T>(0.5);
    let dense = fit_of(fit_ridge(&kept, &kept_y, &RidgeConfig::new(penalty)));

    let streamed = fit_of(fit_ridge_streaming(
        stream_rows(&all, &response)
            .into_iter()
            .filter(|(_, yi)| *yi != sentinel),
        &RidgeConfig::new(penalty),
        2,
    ));

    for (a, b) in dense.beta.iter().zip(streamed.beta.iter()) {
        assert!(
            (*a - *b).abs() <= lift::<T>(tol),
            "filtering in the stream must match materialising the kept rows"
        );
    }
    assert!((dense.sigma2 - streamed.sigma2).abs() <= lift::<T>(tol));
}

#[test]
fn test_fit_ridge_streaming_agrees_over_a_filtered_design() {
    check_forms_agree_when_filtered::<f32>(F32.solve);
    check_forms_agree_when_filtered::<f64>(F64.solve);
    check_forms_agree_when_filtered::<Float106>(F106.solve);
}

// ---------------------------------------------------------------------------------------------
// 22-25. Streaming-form corner cases.
// ---------------------------------------------------------------------------------------------

/// An empty stream is the streaming form's version of an empty design (corner row A).
fn check_streaming_empty<T: RealField + FromPrimitive>() {
    let rows: Vec<(Vec<T>, T)> = Vec::new();
    let err = err_of(fit_ridge_streaming(rows, &RidgeConfig::new(T::one()), 2));
    assert!(matches!(err, StatsErrorEnum::EmptyInput(_)));
}

#[test]
fn test_fit_ridge_streaming_rejects_empty_stream() {
    check_streaming_empty::<f32>();
    check_streaming_empty::<f64>();
    check_streaming_empty::<Float106>();
}

/// A row whose width disagrees with the declared `columns`. Both directions: a short row, whose
/// missing entries have no defined value, and a long one, whose extra entries have no column.
fn check_streaming_row_width<T: RealField + FromPrimitive>() {
    let short: Vec<(Vec<T>, T)> = vec![
        (lift_array::<T>(&[1.0, 2.0]), lift::<T>(1.0)),
        (lift_array::<T>(&[3.0]), lift::<T>(2.0)),
    ];
    let err = err_of(fit_ridge_streaming(short, &RidgeConfig::new(T::one()), 2));
    assert!(matches!(err, StatsErrorEnum::DimensionMismatch(_)));

    let long: Vec<(Vec<T>, T)> = vec![
        (lift_array::<T>(&[1.0, 2.0]), lift::<T>(1.0)),
        (lift_array::<T>(&[3.0, 4.0, 5.0]), lift::<T>(2.0)),
    ];
    let err = err_of(fit_ridge_streaming(long, &RidgeConfig::new(T::one()), 2));
    assert!(matches!(err, StatsErrorEnum::DimensionMismatch(_)));
}

#[test]
fn test_fit_ridge_streaming_rejects_row_width_disagreement() {
    check_streaming_row_width::<f32>();
    check_streaming_row_width::<f64>();
    check_streaming_row_width::<Float106>();
}

/// `columns = 0`: the streaming form's zero-width design (corner row D).
///
/// Ambiguity, and the reading asserted: as in `check_zero_width_design`, the docstrings do not
/// choose between `EmptyInput` and `DimensionMismatch` for a design with no columns, so the test
/// accepts either and rejects a success.
fn check_streaming_zero_columns<T: RealField + FromPrimitive>() {
    let rows: Vec<(Vec<T>, T)> = vec![(Vec::new(), lift::<T>(1.0)), (Vec::new(), lift::<T>(2.0))];
    let err = err_of(fit_ridge_streaming(rows, &RidgeConfig::new(T::one()), 0));
    assert!(matches!(
        err,
        StatsErrorEnum::EmptyInput(_) | StatsErrorEnum::DimensionMismatch(_)
    ));
}

#[test]
fn test_fit_ridge_streaming_rejects_zero_columns() {
    check_streaming_zero_columns::<f32>();
    check_streaming_zero_columns::<f64>();
    check_streaming_zero_columns::<Float106>();
}

/// The streaming form refuses a rank-deficient design at zero penalty on the same terms as the
/// materialised one, and accepts it once the penalty is positive — the same duplicated-column
/// design, and the same hand-solved 14/29 above the threshold, as
/// `check_rank_deficient`. Both sides of the documented `λ = 0` threshold (corner row E).
fn check_streaming_rank_deficient<T: RealField + FromPrimitive>(tol: f64) {
    let x = design::<T>(&[&[1.0, 1.0], &[2.0, 2.0], &[3.0, 3.0]]);
    let y = lift_array::<T>(&[1.0, 2.0, 3.0]);

    let err = err_of(fit_ridge_streaming(
        stream_rows(&x, &y),
        &RidgeConfig::new(T::zero()),
        2,
    ));
    assert!(matches!(err, StatsErrorEnum::RankDeficient(_)));

    let fit = fit_of(fit_ridge_streaming(
        stream_rows(&x, &y),
        &RidgeConfig::new(T::one()),
        2,
    ));
    assert_close(fit.beta[0], 0.4827586206896552, tol, "14/29, streamed");
    assert_close(fit.beta[1], 0.4827586206896552, tol, "14/29, streamed");
}

#[test]
fn test_fit_ridge_streaming_rank_deficient_design() {
    check_streaming_rank_deficient::<f32>(F32.literal);
    check_streaming_rank_deficient::<f64>(F64.literal);
    check_streaming_rank_deficient::<Float106>(F106.literal);
}

// ---------------------------------------------------------------------------------------------
// The elimination itself
// ---------------------------------------------------------------------------------------------
//
// Every fixture above is one or two columns, and its normal matrix needs no row swap, so the
// partial-pivot branch, the swap, and the back-substitution over more than two unknowns were
// reached but never discriminated: mutation testing left twenty survivors inside `solve_symmetric`.
// These three cases exercise it.
//
// The design is six rows by four columns, chosen so the elimination performs exactly one row swap
// at the first column. Expected values are derived in
// `openspec/changes/archive/2026-09-08-unified-math-next/notes/c2-oracles/ridge_pivoting.py`, which solves
// `(XᵀX + λI)β = Xᵀy` over `fractions.Fraction` — exact rational arithmetic, sharing no code with
// the Rust — and reports the swap count.

/// The six-by-four design whose normal matrix needs a pivot swap.
const PIVOT_X: [[f64; 4]; 6] = [
    [1.0, 2.0, 0.0, 1.0],
    [0.0, 1.0, 3.0, 1.0],
    [2.0, 0.0, 1.0, 4.0],
    [1.0, 1.0, 1.0, 1.0],
    [3.0, 1.0, 0.0, 2.0],
    [0.0, 2.0, 2.0, 0.0],
];

/// `y = Xβ` for `β = (2, −1, 3, −2)`, so the zero-penalty fit must return that β.
const PIVOT_Y: [f64; 6] = [-2.0, 6.0, -1.0, 2.0, 1.0, 4.0];

fn pivot_design<T: RealField + FromPrimitive>() -> Vec<Vec<T>> {
    PIVOT_X.iter().map(|r| lift_array::<T>(r)).collect()
}

/// At zero penalty the fit recovers its generator, through an elimination that swaps a row.
fn check_pivoting_recovers_the_generator<T: RealField + FromPrimitive>(tol: f64) {
    let x = pivot_design::<T>();
    let y = lift_array::<T>(&PIVOT_Y);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(T::zero())));

    // The generator, recovered exactly: the oracle reports 2, −1, 3, −2 as exact integers.
    assert_close(fit.beta[0], 2.0, tol, "pivoting beta_1");
    assert_close(fit.beta[1], -1.0, tol, "pivoting beta_2");
    assert_close(fit.beta[2], 3.0, tol, "pivoting beta_3");
    assert_close(fit.beta[3], -2.0, tol, "pivoting beta_4");
}

/// At a positive penalty the fit matches the exact rational solution.
///
/// This is what discriminates the elimination's arithmetic. The zero-penalty case above passes for
/// any method that solves the system, because the answer is the generator; here the answer is a
/// ratio of six-digit integers that only the right elimination produces.
fn check_pivoting_matches_the_exact_penalised_solution<T: RealField + FromPrimitive>(tol: f64) {
    let x = pivot_design::<T>();
    let y = lift_array::<T>(&PIVOT_Y);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(1.0))));

    // λ = 1, from the oracle: 1653/2329, −512/2329, 5003/2329, −4507/4658.
    assert_close(
        fit.beta[0],
        0.709_746_672_391_584,
        tol,
        "lambda=1 beta_1 = 1653/2329",
    );
    assert_close(
        fit.beta[1],
        -0.219_836_839_845_427,
        tol,
        "lambda=1 beta_2 = -512/2329",
    );
    assert_close(
        fit.beta[2],
        2.148_132_245_598_97,
        tol,
        "lambda=1 beta_3 = 5003/2329",
    );
    assert_close(
        fit.beta[3],
        -0.967_582_653_499_356,
        tol,
        "lambda=1 beta_4 = -4507/4658",
    );
}

/// A second penalty, so the penalty is read rather than merely present.
fn check_pivoting_at_a_larger_penalty<T: RealField + FromPrimitive>(tol: f64) {
    let x = pivot_design::<T>();
    let y = lift_array::<T>(&PIVOT_Y);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(4.0))));

    // λ = 4, from the oracle: 843/10807, 2107/10807, 16076/10807, −4130/10807.
    assert_close(
        fit.beta[0],
        0.078_004_996_761_358,
        tol,
        "lambda=4 beta_1 = 843/10807",
    );
    assert_close(
        fit.beta[1],
        0.194_966_225_594_522,
        tol,
        "lambda=4 beta_2 = 2107/10807",
    );
    assert_close(
        fit.beta[2],
        1.487_554_362_912_927,
        tol,
        "lambda=4 beta_3 = 16076/10807",
    );
    assert_close(
        fit.beta[3],
        -0.382_159_711_298_233,
        tol,
        "lambda=4 beta_4 = -4130/10807",
    );
}

#[test]
fn test_fit_ridge_pivoting_recovers_the_generator() {
    check_pivoting_recovers_the_generator::<f32>(F32.solve);
    check_pivoting_recovers_the_generator::<f64>(F64.solve);
    check_pivoting_recovers_the_generator::<Float106>(F106.solve);
}

#[test]
fn test_fit_ridge_pivoting_matches_the_exact_penalised_solution() {
    check_pivoting_matches_the_exact_penalised_solution::<f32>(F32.solve);
    check_pivoting_matches_the_exact_penalised_solution::<f64>(F64.literal);
    check_pivoting_matches_the_exact_penalised_solution::<Float106>(F106.literal);
}

#[test]
fn test_fit_ridge_pivoting_at_a_larger_penalty() {
    check_pivoting_at_a_larger_penalty::<f32>(F32.solve);
    check_pivoting_at_a_larger_penalty::<f64>(F64.literal);
    check_pivoting_at_a_larger_penalty::<Float106>(F106.literal);
}

// ---------------------------------------------------------------------------------------------
// Each half of a compound guard
// ---------------------------------------------------------------------------------------------
//
// A guard written `a || b` is only pinned by a case where exactly one of `a` and `b` holds. With
// both or neither, `&&` behaves identically and the mutant survives. These cases supply the
// asymmetric half for each compound guard in this module.

/// A ragged design, where the column count is non-zero but the rows disagree.
///
/// The guard is `p == 0 || rows disagree`. This holds only the right half.
#[test]
fn test_fit_ridge_ragged_rows_alone_are_refused() {
    let x: Vec<Vec<f64>> = vec![vec![1.0, 2.0], vec![3.0]];
    let y = vec![1.0, 2.0];
    let variant = err_of(fit_ridge(&x, &y, &RidgeConfig::new(0.0)));
    assert!(
        matches!(variant, StatsErrorEnum::DimensionMismatch(_)),
        "a ragged design is a dimension mismatch, got {variant:?}"
    );
}

/// A zero-column design whose rows all agree — the left half of the same guard, alone.
#[test]
fn test_fit_ridge_zero_columns_with_consistent_rows_are_refused() {
    let x: Vec<Vec<f64>> = vec![vec![], vec![]];
    let y = vec![1.0, 2.0];
    let variant = err_of(fit_ridge(&x, &y, &RidgeConfig::new(0.0)));
    assert!(
        matches!(variant, StatsErrorEnum::DimensionMismatch(_)),
        "a design with no columns has nothing to fit, got {variant:?}"
    );
}

/// A non-finite entry in the DESIGN with a finite response.
///
/// The guard is `row non-finite || response non-finite`. This holds only the left half.
#[test]
fn test_fit_ridge_non_finite_design_with_finite_response() {
    let x: Vec<Vec<f64>> = vec![vec![1.0, f64::NAN], vec![1.0, 2.0]];
    let y = vec![1.0, 2.0];
    let variant = err_of(fit_ridge(&x, &y, &RidgeConfig::new(0.0)));
    assert!(
        matches!(variant, StatsErrorEnum::NonFiniteInput(_)),
        "a NaN in the design is refused even when every response is finite, got {variant:?}"
    );
}

/// A non-finite RESPONSE with a finite design — the right half, alone.
#[test]
fn test_fit_ridge_finite_design_with_non_finite_response() {
    let x: Vec<Vec<f64>> = vec![vec![1.0, 1.0], vec![1.0, 2.0]];
    let y = vec![1.0, f64::INFINITY];
    let variant = err_of(fit_ridge(&x, &y, &RidgeConfig::new(0.0)));
    assert!(
        matches!(variant, StatsErrorEnum::NonFiniteInput(_)),
        "an infinite response is refused even when the design is clean, got {variant:?}"
    );
}

/// The pivot search selects the row of largest magnitude, and a swap actually moves it.
///
/// Written as a design whose first normal-matrix column has its largest entry off the diagonal, so
/// the search does move a row and the swap machinery runs on a real case.
///
/// What it does not pin is the choice. The diagonal here is intact, and an elimination reaches the
/// same coefficients through any non-zero pivot — a worse-conditioned route, not a wrong one — so
/// this design cannot separate a pivot search from a weakened one. That is the job of
/// `test_fit_ridge_pivot_swap_the_answer_depends_on` below, which empties a diagonal entry so the
/// unswapped route divides by zero.
#[test]
fn test_fit_ridge_pivot_search_finds_the_largest_magnitude_row() {
    // Two columns whose normal matrix is [[1, 3], [3, 10]]: the first column's largest entry is
    // the off-diagonal 3, so column 0 must swap in row 1 before eliminating.
    let x: Vec<Vec<f64>> = vec![vec![1.0, 3.0], vec![0.0, 1.0]];
    // y = X · (2, -1) = (2·1 + 3·(-1), 2·0 + 1·(-1)) = (-1, -1)
    let y = vec![-1.0, -1.0];
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(0.0)));
    assert_close(fit.beta[0], 2.0, F64.solve, "pivot search beta_1");
    assert_close(fit.beta[1], -1.0, F64.solve, "pivot search beta_2");
}

/// A swap the answer depends on: a negative penalty that empties a diagonal entry.
///
/// The test above pins the search on a matrix whose diagonal is intact, where a skipped swap still
/// divides by something and reaches the same coefficients by a worse-conditioned route. That makes
/// it a regression for the swap machinery but not for the choice, so it cannot separate a pivot
/// search from a broken one.
///
/// This design can. `XᵀX + λI` is symmetric, and for `λ ≥ 0` it is also positive semi-definite,
/// where `|a_ij|² ≤ a_ii a_jj` forces a zero on the diagonal to carry a zero column with it — so no
/// non-negative penalty can produce a pivot that must be moved. A negative penalty is not bound by
/// that, and the crate accepts one: it subtracts from the diagonal while the off-diagonal stands.
/// At `λ = −1` this design gives `[[0, 2], [2, 5]]`, whose first pivot is zero and whose first
/// column is not. A solve that does not move a row divides by that zero and returns
/// `RankDeficient`, so the fit is lost outright rather than merely losing accuracy.
///
/// This is now a test of the delegation as much as of the arithmetic: the solve is
/// `deep_causality_linear`'s LU, and what this pins is that ridge hands it a system it can factor
/// and maps its failure back onto the right `StatsError`. An indefinite normal matrix is the case a
/// Cholesky would refuse, which is why the delegation names LU.
///
/// The answer is the exact rational solution of those normal equations, evaluated as fractions
/// rather than reproduced from this crate: `β = (3/4, 1/2)` and `σ² = rss/dof = (17/16)/1`.
fn check_pivot_swap_the_answer_depends_on<T: RealField + FromPrimitive>(tol: f64) {
    let x = design::<T>(&[&[1.0, 2.0], &[0.0, 1.0], &[0.0, 1.0]]);
    let y = lift_array::<T>(&[1.0, 1.0, 1.0]);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(-1.0))));
    assert_close(fit.beta[0], 0.75, tol, "swap-dependent beta_1");
    assert_close(fit.beta[1], 0.5, tol, "swap-dependent beta_2");
    assert_close(fit.sigma2, 1.0625, tol, "swap-dependent sigma2");
}

#[test]
fn test_fit_ridge_pivot_swap_the_answer_depends_on() {
    check_pivot_swap_the_answer_depends_on::<f32>(F32.solve);
    check_pivot_swap_the_answer_depends_on::<f64>(F64.solve);
    check_pivot_swap_the_answer_depends_on::<Float106>(F106.solve);
}

/// The same dependence one column in, where the row and column strides are no longer equal.
///
/// A swap at column 0 exercises the least of any indexing: the column term is zero, so a stride
/// that added it, subtracted it or dropped it entirely all address the same element. This design
/// leaves column 0's pivot on the diagonal and puts the swap at column 1, where that term has to
/// carry. `XᵀX + λI` is `[[4, 2, 3], [2, 0, 0], [3, 0, 10]]` at `λ = −1`; eliminating column 0
/// leaves `−1` on the diagonal against `−3/2` below it, so column 1 swaps.
///
/// The response is scaled so the answer is exact in binary. `β` is linear in `y`, so the natural
/// response `(1, 2, 1, 1)` and its exact `β = (1/2, −7/40, 9/20)` scale by 40 into
/// `β = (20, −7, 18)`, which every shipped scalar holds without rounding. The unscaled thirds and
/// fortieths do not: carried as `f64` literals they are already wrong by about 1e-18, which is
/// visible against `Float106`'s tolerance and would make the assertion a statement about the
/// literal rather than about the solve.
///
/// `σ²` scales by 40² to 5533 and is not asserted here. At `f32` that magnitude costs about 6.6e-4
/// in representation alone, past the absolute tolerance this row allows, and the swap-dependent
/// `σ²` is already pinned by the two-column test above.
fn check_pivot_swap_at_the_second_column<T: RealField + FromPrimitive>(tol: f64) {
    let x = design::<T>(&[
        &[2.0, 1.0, 0.0],
        &[0.0, 0.0, 1.0],
        &[0.0, 0.0, 1.0],
        &[1.0, 0.0, 3.0],
    ]);
    let y = lift_array::<T>(&[40.0, 80.0, 40.0, 40.0]);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(-1.0))));
    assert_close(fit.beta[0], 20.0, tol, "second-column swap beta_1");
    assert_close(fit.beta[1], -7.0, tol, "second-column swap beta_2");
    assert_close(fit.beta[2], 18.0, tol, "second-column swap beta_3");
}

#[test]
fn test_fit_ridge_pivot_swap_at_the_second_column() {
    check_pivot_swap_at_the_second_column::<f32>(F32.solve);
    check_pivot_swap_at_the_second_column::<f64>(F64.solve);
    check_pivot_swap_at_the_second_column::<Float106>(F106.solve);
}

/// A swap where the solve has to reach past the next row, at four columns.
///
/// The narrower fixtures above leave two ways for a nearly-right pivot search to pass anyway. At
/// three columns the read `row * n - col` for row 2, column 1 is slot 5, and so is the transpose
/// `col * n + row`; because the Schur complement of a symmetric matrix is symmetric, a search that
/// addressed the transpose would find the same number. And their column-1 pivot is only zero
/// *before* column 0 is eliminated — afterwards it is −1, so a search landing elsewhere still
/// divides by something non-zero and recovers.
///
/// Four columns break the coincidence, and this design breaks the recovery. At `λ = −3` the normal
/// matrix is `[[27, −9, 9, 9], [−9, 3, −2, −3], [9, −2, 8, 5], [9, −3, 5, 21]]`, whose first column
/// pivots on the diagonal. Eliminating it leaves column 1 as `(0, 1, 0)` from the diagonal down:
/// the diagonal is empty, the row directly below it is empty, and the column's only pivot is two
/// rows away. A solve that stops before reaching it divides by zero and returns `RankDeficient`.
///
/// This outlives the local elimination it was built against. The solve is now
/// `deep_causality_linear`'s LU, and a design whose pivot is neither on the diagonal nor in the
/// next row is worth keeping against any factorisation ridge delegates to.
///
/// Verified against the elimination in `notes/c2-oracles/ridge_pivoting.py`, which reruns the pivot
/// search under each mis-addressed index in exact arithmetic: the true index returns
/// `β = (−4, −5, 3, 1)` and every mis-addressed one returns `RankDeficient`.
fn check_pivot_reaches_past_the_next_row<T: RealField + FromPrimitive>(tol: f64) {
    let x = design::<T>(&[
        &[-2.0, 0.0, -1.0, -1.0],
        &[2.0, -1.0, -2.0, -1.0],
        &[3.0, -1.0, 2.0, -2.0],
        &[2.0, -2.0, 1.0, 3.0],
        &[3.0, 0.0, 1.0, 3.0],
    ]);
    let y = lift_array::<T>(&[-6.0, -6.0, -6.0, 0.0, -3.0]);
    let fit = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(lift::<T>(-3.0))));
    assert_close(fit.beta[0], -4.0, tol, "reaching pivot beta_1");
    assert_close(fit.beta[1], -5.0, tol, "reaching pivot beta_2");
    assert_close(fit.beta[2], 3.0, tol, "reaching pivot beta_3");
    assert_close(fit.beta[3], 1.0, tol, "reaching pivot beta_4");
}

#[test]
fn test_fit_ridge_pivot_reaches_past_the_next_row() {
    check_pivot_reaches_past_the_next_row::<f32>(F32.solve);
    check_pivot_reaches_past_the_next_row::<f64>(F64.solve);
    check_pivot_reaches_past_the_next_row::<Float106>(F106.solve);
}

/// The exempt column is not shrunk, on a design where the difference is exactly checkable.
///
/// An intercept-only design reduces the normal equations to a scalar: `XᵀX = n`, `Xᵀy = Σy`, so
/// `β = Σy / (n + λ)` when the penalty reaches the column and `β = Σy / n` when it does not. With
/// `y = (1, 2, 3, 4)` and `λ = 1` that is `10/5 = 2` against `10/4 = 5/2` — both exact in binary,
/// and far enough apart that no tolerance question arises.
///
/// The exempt fit is the sample mean, which is the point of the convention: the intercept is free
/// to carry the level of `y`, and the penalty acts only on the shape.
fn check_penalisation_exempts_a_column<T: RealField + FromPrimitive>(tol: f64) {
    let x = design::<T>(&[&[1.0], &[1.0], &[1.0], &[1.0]]);
    let y = lift_array::<T>(&[1.0, 2.0, 3.0, 4.0]);
    let penalty = lift::<T>(1.0);

    let all = fit_of(fit_ridge(&x, &y, &RidgeConfig::new(penalty)));
    assert_close(all.beta[0], 2.0, tol, "penalised intercept");

    let exempt = fit_of(fit_ridge(
        &x,
        &y,
        &RidgeConfig::new(penalty).with_penalisation(Penalisation::Excluding(0)),
    ));
    assert_close(
        exempt.beta[0],
        2.5,
        tol,
        "exempt intercept is the sample mean",
    );
}

#[test]
fn test_fit_ridge_penalisation_exempts_a_column() {
    check_penalisation_exempts_a_column::<f32>(F32.solve);
    check_penalisation_exempts_a_column::<f64>(F64.solve);
    check_penalisation_exempts_a_column::<Float106>(F106.solve);
}

/// Exempting a column the design does not have is a caller error, not a silent no-op: it means the
/// caller believes the intercept sits somewhere it does not.
#[test]
fn test_fit_ridge_exempt_column_outside_the_design_is_refused() {
    let x = design::<f64>(&[&[1.0, 2.0], &[1.0, 3.0], &[1.0, 4.0]]);
    let y = lift_array::<f64>(&[1.0, 2.0, 3.0]);
    let err = err_of(fit_ridge(
        &x,
        &y,
        &RidgeConfig::new(1.0).with_penalisation(Penalisation::Excluding(2)),
    ));
    assert!(
        matches!(err, StatsErrorEnum::DimensionMismatch(_)),
        "an exempt column past the design width is a shape error, got {err:?}"
    );
}
