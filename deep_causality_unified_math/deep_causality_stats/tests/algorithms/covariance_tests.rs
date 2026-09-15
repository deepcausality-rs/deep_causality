/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Suite for `column_means`, `covariance_matrix` and `conditional_variance`.
//!
//! # Where the expected values come from
//!
//! Every expectation here is one of:
//!
//! * a **hand-evaluated closed form** on a fixture small enough to write the arithmetic out beside
//!   it — three observations of two variables, and a two-by-two Schur complement;
//! * an **algebraic invariant**: the covariance matrix is symmetric, its diagonal is the per-column
//!   variance, and the conditional variance of a target on a set of parents is a function of that
//!   *set* rather than of the list that spells it;
//! * an **arithmetic fact about `usize`**, for the shapes that cannot exist.
//!
//! None is produced by calling the function under test, and none retypes the implementation's
//! formula.
//!
//! # What this suite is for
//!
//! Three shapes reach these functions from a caller rather than from a slice, and none of them is
//! bounded by anything the arithmetic can rely on: the observation count, the variable count, and
//! the parent list. Two of the three could previously turn into a wrong answer rather than a
//! refusal — a stated shape whose cell count overflows a `usize`, and a parent list that is not a
//! set — and the tests for those are the last two sections.

use deep_causality_algebra::RealField;
use deep_causality_num::{BFloat16, Float106, FromPrimitive, lift};
use deep_causality_stats::utils_tests::lift_array;
use deep_causality_stats::utils_tests::precision::{BF16, F32, F64, F106};
use deep_causality_stats::{
    StatsError, StatsErrorEnum, column_means, conditional_variance, covariance_matrix, mean,
    variance,
};

fn err_of<T>(r: Result<T, StatsError>) -> StatsErrorEnum {
    match r {
        Ok(_) => panic!("expected a typed error"),
        Err(e) => e.0,
    }
}

fn assert_close<T: RealField + FromPrimitive>(actual: T, expected: f64, tol: f64, what: &str) {
    let a = actual.to_f64().unwrap_or(f64::NAN);
    let scale = expected.abs().max(1.0);
    assert!(
        (a - expected).abs() <= tol * scale,
        "{what}: got {a}, want {expected}"
    );
}

// ---------------------------------------------------------------------------------------------
// Closed forms, so the refusals below are refusals and not a function that never worked
// ---------------------------------------------------------------------------------------------

/// Provenance: closed forms by hand on the three-by-two matrix
///
/// ```text
/// x = [[1, 2],
///      [3, 4],
///      [5, 6]]
/// ```
///
/// Column means: `(1 + 3 + 5)/3 = 3` and `(2 + 4 + 6)/3 = 4`.
///
/// Deviations: both columns are `[−2, 0, 2]`, so every entry of the corrected covariance matrix is
/// `((−2)(−2) + 0 + (2)(2)) / (3 − 1) = 8/2 = 4`. The two columns are equal up to a shift, so the
/// matrix is `[[4, 4], [4, 4]]` — symmetric, with the per-column variance on its diagonal.
fn check_closed_forms<T: RealField + FromPrimitive>(tol: f64) {
    let data = lift_array::<T>(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);

    let means = column_means(&data, 3, 2).expect("a three-by-two matrix has column means");
    assert_eq!(means.len(), 2, "one mean per variable");
    assert_close(means[0], 3.0, tol, "the first column mean");
    assert_close(means[1], 4.0, tol, "the second column mean");

    let cov = covariance_matrix(&data, 3, 2).expect("three observations admit a covariance");
    assert_eq!(cov.len(), 4, "a two-variable covariance is two by two");
    for (i, entry) in cov.iter().enumerate() {
        assert_close(*entry, 4.0, tol, &alloc_label(i));
    }
    assert!(
        cov[1] == cov[2],
        "the covariance matrix is symmetric: {:?} against {:?}",
        cov[1].to_f64(),
        cov[2].to_f64()
    );
}

fn alloc_label(i: usize) -> String {
    format!("covariance entry {i}")
}

#[test]
fn test_column_means_and_covariance_closed_forms() {
    check_closed_forms::<f32>(F32.native);
    check_closed_forms::<f64>(F64.native);
    check_closed_forms::<Float106>(F106.literal);
}

/// Provenance: the Schur complement evaluated by hand.
///
/// With
///
/// ```text
/// Σ = [[2, 1],
///      [1, 3]]
/// ```
///
/// the variance of variable 0 given variable 1 is
/// `Σ_yy − Σ_yP Σ_PP⁻¹ Σ_Py = 2 − 1 · (1/3) · 1 = 2 − 1/3 = 5/3 = 1.6666...`.
///
/// An empty parent set leaves the marginal variance `Σ_yy = 2`, which the function's own doc
/// states.
fn check_conditional_variance_closed_form<T: RealField + FromPrimitive>(tol: f64) {
    let cov = lift_array::<T>(&[2.0, 1.0, 1.0, 3.0]);

    let marginal = conditional_variance(&cov, 2, 0, &[], lift::<T>(0.0))
        .expect("an empty parent set leaves the marginal variance");
    assert_close(marginal, 2.0, tol, "the marginal variance");

    let conditioned = conditional_variance(&cov, 2, 0, &[1], lift::<T>(0.0))
        .expect("one parent, and the block is invertible");
    // 5/3 = 1.6666666666666667
    assert_close(conditioned, 5.0 / 3.0, tol, "the Schur complement");
}

#[test]
fn test_conditional_variance_closed_form() {
    check_conditional_variance_closed_form::<f32>(F32.native);
    check_conditional_variance_closed_form::<f64>(F64.native);
    check_conditional_variance_closed_form::<Float106>(F106.literal);
}

// ---------------------------------------------------------------------------------------------
// A stated shape with no cell count
// ---------------------------------------------------------------------------------------------

/// Provenance: an arithmetic fact about `usize`, and this crate's refusal contract for a shape
/// that does not exist.
///
/// The observation and variable counts are the caller's, and neither is read off the slice. Their
/// product is the cell count the data length is checked against, and `usize::MAX · 4` has no
/// `usize` value: an unchecked multiply panics on overflow in a debug build and wraps in a release
/// one. A wrapped product is the worse of the two — `2⁶² · 4` wraps to zero, which *matches* an
/// empty slice, and the shape is then accepted and indexed row by row off the end of it.
///
/// A shape whose product exists but does not match the data is asserted beside it, so the refusal
/// is shown to be about the arithmetic and not about every large count.
#[test]
#[cfg(target_pointer_width = "64")]
fn test_a_shape_whose_cell_count_has_no_usize_is_refused() {
    let data = [1.0_f64, 2.0];
    let empty: [f64; 0] = [];

    for (observations, variables) in [
        (usize::MAX, 4usize),
        (1usize << 62, 4),
        (1usize << 40, 1 << 40),
    ] {
        let err = err_of(column_means(&data, observations, variables));
        assert!(
            matches!(err, StatsErrorEnum::DimensionMismatch(_)),
            "column_means({observations}, {variables}): expected DimensionMismatch, got {err:?}"
        );

        let err = err_of(covariance_matrix(&data, observations, variables));
        assert!(
            matches!(err, StatsErrorEnum::DimensionMismatch(_)),
            "covariance_matrix({observations}, {variables}): expected DimensionMismatch, got {err:?}"
        );

        // The wrapping product `2^62 · 4 = 0` matches an empty slice, so the empty case is where a
        // wrapped count would be accepted rather than merely mis-reported.
        let err = err_of(column_means(&empty, observations, variables));
        assert!(
            matches!(err, StatsErrorEnum::DimensionMismatch(_)),
            "column_means over an empty slice at ({observations}, {variables}): got {err:?}"
        );
    }

    // `conditional_variance` states one count and squares it.
    let err = err_of(conditional_variance(&data, 1usize << 32, 0, &[1], 0.0_f64));
    assert!(
        matches!(err, StatsErrorEnum::DimensionMismatch(_)),
        "conditional_variance at 2^32 variables: expected DimensionMismatch, got {err:?}"
    );

    // A product that exists and simply does not match: the ordinary shape mismatch, unchanged.
    let err = err_of(column_means(&data, 3, 2));
    assert!(
        matches!(err, StatsErrorEnum::DimensionMismatch(_)),
        "a length that is not the product is still a shape mismatch, got {err:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// The parents are a set
// ---------------------------------------------------------------------------------------------

/// Provenance: an algebraic invariant — the residual variance of a target after conditioning on a
/// set of parents is a function of that set. Naming a variable twice does not add a variable, so
/// the answer must not depend on how many times it was named.
///
/// The implementation cannot honour that invariant by computing, because `Σ_PP` for a repeated
/// parent has a duplicated row and column and is exactly singular. At `ridge = 0` that is refused
/// as collinearity, which is the right answer for the wrong reason. At any positive ridge the
/// duplicated block is *solvable*, and it answers something other than the same parent once: on
/// the fixture below, `1.6721` against `1.6774`. Two spellings of one parent set, two numbers.
///
/// So the list is required to be a set, and the refusal is `DimensionMismatch` — the same variant
/// an out-of-range index gets, because both are the caller having described a set of variables
/// that does not exist.
fn check_repeated_parent_is_refused<T: RealField + FromPrimitive>() {
    let cov = lift_array::<T>(&[2.0, 1.0, 1.0, 3.0]);

    for ridge in [0.0, 0.1, 1.0] {
        let err = err_of(conditional_variance(&cov, 2, 0, &[1, 1], lift::<T>(ridge)));
        assert!(
            matches!(err, StatsErrorEnum::DimensionMismatch(_)),
            "a parent listed twice at ridge {ridge}: expected DimensionMismatch, got {err:?}"
        );
    }

    // Three parents in a three-variable matrix, with the repeat away from the front.
    let cov3 = lift_array::<T>(&[2.0, 1.0, 0.5, 1.0, 3.0, 0.25, 0.5, 0.25, 4.0]);
    let err = err_of(conditional_variance(
        &cov3,
        3,
        0,
        &[1, 2, 1],
        lift::<T>(0.1),
    ));
    assert!(
        matches!(err, StatsErrorEnum::DimensionMismatch(_)),
        "a repeat in the third position: expected DimensionMismatch, got {err:?}"
    );

    // The set it spells is accepted, so the refusal is about the repeat and not about the parents.
    let ok = conditional_variance(&cov3, 3, 0, &[1, 2], lift::<T>(0.1))
        .expect("two distinct parents are a set");
    assert!(
        ok.to_f64().unwrap_or(f64::NAN).is_finite(),
        "the deduplicated set has an answer"
    );
}

#[test]
fn test_repeated_parent_is_refused() {
    check_repeated_parent_is_refused::<f32>();
    check_repeated_parent_is_refused::<f64>();
    check_repeated_parent_is_refused::<Float106>();
}

/// Provenance: the definition. The residual variance of a variable after regressing it on itself
/// is zero — the regression is exact — so there is nothing left to estimate and no parent set for
/// which this is a question.
///
/// It is refused rather than answered because the answer the arithmetic gives is not that zero.
/// At `ridge = 0` the Schur complement is `Σ_yy − Σ_yy · Σ_yy⁻¹ · Σ_yy = 0`, which is right; at a
/// positive ridge it is `Σ_yy − Σ_yy²/(Σ_yy + λ)`, which is strictly positive — `0.4` at
/// `Σ_yy = 2, λ = 0.5`, by hand — and a strictly positive residual variance for a variable given
/// itself is a number that means nothing.
fn check_target_among_its_own_parents_is_refused<T: RealField + FromPrimitive>() {
    let cov = lift_array::<T>(&[2.0, 1.0, 1.0, 3.0]);

    for ridge in [0.0, 0.5] {
        let err = err_of(conditional_variance(&cov, 2, 0, &[0], lift::<T>(ridge)));
        assert!(
            matches!(err, StatsErrorEnum::DimensionMismatch(_)),
            "the target as its own only parent at ridge {ridge}: got {err:?}"
        );

        let err = err_of(conditional_variance(&cov, 2, 0, &[0, 1], lift::<T>(ridge)));
        assert!(
            matches!(err, StatsErrorEnum::DimensionMismatch(_)),
            "the target among its parents at ridge {ridge}: got {err:?}"
        );

        // The other target, so the check is on the target given and not on index zero.
        let err = err_of(conditional_variance(&cov, 2, 1, &[0, 1], lift::<T>(ridge)));
        assert!(
            matches!(err, StatsErrorEnum::DimensionMismatch(_)),
            "target 1 among its parents at ridge {ridge}: got {err:?}"
        );
    }
}

#[test]
fn test_target_among_its_own_parents_is_refused() {
    check_target_among_its_own_parents_is_refused::<f32>();
    check_target_among_its_own_parents_is_refused::<f64>();
    check_target_among_its_own_parents_is_refused::<Float106>();
}

// -------------------------------------------------------------------------------------------
// Reach: every entry of the matrix, and every column mean, is a reduction over the rows. A narrow
// scalar over many rows is where an arrangement that loses them shows, and the cases below assert
// structural facts — symmetry, agreement with the one-dimensional statistics — rather than values,
// so they hold at any precision that can represent the answer.
// -------------------------------------------------------------------------------------------

/// Row K. The matrix is symmetric by construction, at every count and scalar.
///
/// `cov[i][j]` and `cov[j][i]` are sums of the same products, so an arrangement that loses either
/// must lose both identically for this to hold. It is the cheapest structural check available and
/// it costs nothing to run at a thousand rows, where the reductions are long.
#[test]
fn test_the_covariance_matrix_is_symmetric_over_many_rows() {
    let (data, n, p) = wide_fixture::<BFloat16>(1000);
    let cov = covariance_matrix(&data, n, p).expect("a thousand rows are enough");

    for i in 0..p {
        for j in 0..p {
            assert_eq!(
                cov[i * p + j],
                cov[j * p + i],
                "cov[{i}][{j}] and cov[{j}][{i}] must be the same sum"
            );
        }
    }
}

/// The diagonal is the variance of its own column, and the column means are the means of their own
/// columns — the same quantities reached by a different route through this crate.
///
/// This is the strongest check the file can make without a literal: `variance` and `mean` reduce a
/// contiguous slice while the matrix reduces a strided one, so the two agree only if both
/// arrangements are right. A stalling total in either shows as a disagreement, and at `BFloat16`
/// over a thousand rows the disagreement was 20%.
#[test]
fn test_the_diagonal_and_the_means_agree_with_the_one_dimensional_statistics() {
    let (data, n, p) = wide_fixture::<BFloat16>(1000);
    let cov = covariance_matrix(&data, n, p).expect("a thousand rows are enough");
    let means = column_means(&data, n, p).expect("a thousand rows are enough");

    for c in 0..p {
        let column: Vec<BFloat16> = (0..n).map(|row| data[row * p + c]).collect();

        assert_eq!(
            means[c],
            mean(&column).expect("a column has a mean"),
            "column {c}: column_means must agree with mean over the same column"
        );
        assert_eq!(
            cov[c * p + c],
            variance(&column).expect("a column has a variance"),
            "column {c}: the diagonal must agree with variance over the same column"
        );
    }
}

/// Row G. Negating a column negates its covariances with every other column and leaves its own
/// variance alone.
///
/// A sign lost inside the accumulation would show as a covariance that did not move, and a sign
/// applied twice as a variance that did.
#[test]
fn test_negating_a_column_negates_its_covariances_and_not_its_variance() {
    let (data, n, p) = wide_fixture::<BFloat16>(512);
    let cov = covariance_matrix(&data, n, p).expect("enough rows");

    let mut flipped = data.clone();
    for row in 0..n {
        flipped[row * p] = -flipped[row * p];
    }
    let cov_flipped = covariance_matrix(&flipped, n, p).expect("enough rows");

    assert_eq!(
        cov_flipped[0], cov[0],
        "a column's own variance is unchanged by negating it"
    );
    for j in 1..p {
        assert_eq!(
            cov_flipped[j], -cov[j],
            "cov[0][{j}] must negate when column 0 does"
        );
        assert_ne!(
            cov[j],
            lift::<BFloat16>(0.0),
            "the fixture must have a non-zero covariance, or the sign is asserted where it vanishes"
        );
    }
}

/// Three columns over `n` rows, at the working scalar: a ramp, a positive affine image of it, and a
/// negative affine image.
///
/// The ramp runs over `0..n` so the deviations are of order `n/4`, far above the scalar's own
/// spacing — a fixture of closely-spaced values would measure the format's resolution rather than
/// the arrangement of the sums.
fn wide_fixture<T: FromPrimitive>(n: usize) -> (Vec<T>, usize, usize) {
    let mut data = Vec::with_capacity(n * 3);
    for i in 0..n {
        let x = i as f64;
        data.push(lift::<T>(x));
        data.push(lift::<T>(2.0 * x + 1.0));
        data.push(lift::<T>(-x + 7.0));
    }
    (data, n, 3)
}

/// A value, not an agreement: the variance of a ramp, in closed form, at the narrowest scalar.
///
/// The structural cases above compare the matrix against `variance` and `mean`. That is worth
/// having — it pins the strided reduction against the contiguous one — but it cannot detect an
/// arrangement that loses the sum, because both sides would lose it identically and still agree.
/// This case pins the number instead.
///
/// Provenance: the corrected variance of `0..n−1` is `n(n+1)/12`. At `n = 1000` that is
/// `1000·1001/12 = 83416.66…`, and `cov(x, 2x + 1) = 2·var(x)` because covariance is bilinear and
/// the constant contributes nothing. Measured before the reductions were summed as trees: 67072
/// against 83417, eleven times the spacing at that magnitude.
#[test]
fn test_the_variance_of_a_ramp_is_its_closed_form_at_the_narrowest_scalar() {
    let n = 1000usize;
    let (data, rows, p) = wide_fixture::<BFloat16>(n);
    let cov = covariance_matrix(&data, rows, p).expect("a thousand rows are enough");

    // n(n+1)/12 = 1000·1001/12.
    let expected_var = n as f64 * (n as f64 + 1.0) / 12.0;
    // `assert_close` here already scales the tolerance by `|expected|`, so the relative figure
    // goes in directly.
    assert_close(
        cov[0],
        expected_var,
        BF16.reduction,
        "var(0..999) against n(n+1)/12",
    );
    // Column 1 is 2x + 1, so cov(x, 2x + 1) = 2·var(x); column 2 is −x + 7, so cov is −var(x).
    assert_close(
        cov[1],
        2.0 * expected_var,
        BF16.reduction,
        "cov(x, 2x + 1) = 2·var(x)",
    );
    assert_close(
        cov[2],
        -expected_var,
        BF16.reduction,
        "cov(x, −x + 7) = −var(x)",
    );
}
