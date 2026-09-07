/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Phase 2 suite for the logistic family: [`sigmoid`] and [`fit_logistic`].
//!
//! Written before the implementation. Every test in this file is expected to fail with the
//! `phase 1: declared, not implemented` panic until the bodies are written.
//!
//! # Where the expected values come from
//!
//! No expected value in this file is produced by calling the function it checks, and none is
//! a retyping of the formula under test. Each numeric literal carries a comment naming its
//! source, and each closed form is written out so the arithmetic can be checked by hand.
//!
//! The one place a formula is written in the test body is the *score residual* oracle used
//! against `fit_logistic`. That is the first-order condition `Xᵀ(p − y) = 0` which *defines*
//! the estimate, evaluated by a plain sum; the function under test computes the estimate by
//! an iteratively reweighted least-squares solve. Definition against algorithm is a different
//! computation, not the same one typed twice — and inside that test the oracle is itself
//! pinned first at a point where it does *not* vanish, against hand arithmetic, so a broken
//! oracle cannot pass by returning zero everywhere.
//!
//! # Readings asserted where the doc is silent
//!
//! * `fit_logistic` fits **one coefficient per column of the design it is handed** and adds no
//!   intercept of its own — `LogisticFit::beta` says "one per column of the design". Every
//!   fixture below that wants an intercept carries an explicit column of ones, and the suite
//!   pins `beta.len()` against the column count.
//! * The count carried by `NotConverged` when the cap is reached **is the cap**: that many
//!   iterations were performed and none met the stopping test.
//! * `sigmoid` is the continuous extension of `1/(1 + e^{−x})` to the extended reals:
//!   `σ(+∞) = 1`, `σ(−∞) = 0`, and a `NaN` argument propagates.
//! * A label outside `{0, 1}` is refused under `NegativeProbability`, the crate's one variant
//!   for an input that is not a valid probability. The enum carries no label-specific variant.
//!
//! # Corner-case enumeration (`openspec/changes/unified-math-next/tdd/corner-cases.md`)
//!
//! | # | Class | Covered by |
//! |---|---|---|
//! | A | Empty input | `fit_logistic_refuses_an_empty_design` — no rows, and rows with no columns |
//! | B | Single element | `fit_logistic_fits_a_single_observation_under_a_penalty` — one row, one column |
//! | C | Two quantities coincide | `sigmoid_at_zero_is_exactly_one_half` (the two branches of a stable sigmoid meet at 0, where σ = 1 − σ); `fit_logistic_refuses_a_rank_deficient_design_at_zero_penalty` (two identical columns) |
//! | D | An index expression degenerates | `fit_logistic_fits_a_single_observation_under_a_penalty` (n = 1, p = 1, so every design index collapses to one entry); the duplicated-column design, where the two Hessian off-diagonals equal both diagonals |
//! | E | Each threshold, both sides | `fit_logistic_reports_non_convergence_with_the_iteration_count` — the same data and tolerance below the cap (typed error) and above it (a fit) |
//! | F | Zero | `sigmoid_at_zero_is_exactly_one_half`; a zero design entry in `OVERLAP_X`; a zero penalty in the score-equation and separable-data tests; `fit_logistic_with_a_zero_iteration_cap_reports_zero_iterations` |
//! | G | Negative | negative design entries in every fixture; a negative label in `fit_logistic_refuses_a_label_outside_the_unit_interval`. A **negative penalty is deliberately not pinned**: the error enum carries no variant for one and no doc names the behaviour, so any assertion would be inventing semantics rather than testing them. |
//! | H | Exact domain boundary | `fit_logistic_refuses_a_label_outside_the_unit_interval` exercises both sides of the label domain — labels of exactly 0 and exactly 1 are accepted and fitted, −1 and 2 are refused |
//! | I | Non-finite | `sigmoid_maps_non_finite_arguments_to_their_limits`; `fit_logistic_refuses_a_non_finite_design_or_label` |
//! | J | Overflow / underflow reach | `sigmoid_saturates_where_exp_overflows` (±800, where `exp` overflows at all three precisions); `sigmoid_saturates_at_the_precisions_own_extreme` (a magnitude near each type's own maximum); `fit_logistic_never_returns_a_non_finite_coefficient` (a design entry whose square is past the type's maximum) |
//! | K | f32, f64 and Float106 | every test below is a generic body called once per precision, each with its own tolerance set (`F32`, `F64`, `F106`) |

use deep_causality_algebra::{Real, RealField};
use deep_causality_num::lift;
use deep_causality_num::{Float106, FromPrimitive};
use deep_causality_stats::utils_tests::lift_array;
use deep_causality_stats::utils_tests::precision::{F32, F64, F106, Prec};

#[derive(Clone, Copy)]
struct Fit {
    /// The `tolerance` argument handed to `fit_logistic`.
    tolerance: f64,
    /// Slack on a quantity exact only at the exact optimum, which the fit reaches only as closely
    /// as `tolerance` allows. `sqrt(20 · tolerance)`: with the stopping test on the change in the
    /// objective, a stopping change of `t` leaves a gradient of order `sqrt(2·H·t)`, and `H` is of
    /// order ten for these fixtures.
    converged: f64,
}

const FIT_F32: Fit = Fit {
    tolerance: 1e-5,
    converged: 5e-2,
};
const FIT_F64: Fit = Fit {
    tolerance: 1e-12,
    converged: 1e-4,
};
const FIT_F106: Fit = Fit {
    tolerance: 1e-20,
    converged: 1e-8,
};

use deep_causality_stats::{
    LogisticConfig, LogisticFit, Penalisation, StatsError, StatsErrorEnum, fit_logistic, sigmoid,
};

// ---------------------------------------------------------------------------
// Small utilities. None of these calls a function under test.
// ---------------------------------------------------------------------------

/// `|a − b| ≤ tol`.
fn close<T: Real>(a: T, b: T, tol: T) -> bool {
    (a - b).abs() <= tol
}

/// The Euclidean norm of a coefficient vector.
fn l2_norm<T: RealField>(v: &[T]) -> T {
    let mut acc = T::zero();
    for value in v {
        acc += *value * *value;
    }
    acc.sqrt()
}

/// Lifts a table of `f64` rows into a design over the working scalar.
fn design<T: FromPrimitive, const N: usize>(rows: &[[f64; N]]) -> Vec<Vec<T>> {
    rows.iter().map(|row| lift_array::<T>(row)).collect()
}

/// Unwraps a fit, reporting the typed error rather than the raw `unwrap` message.
fn expect_fit<T: RealField + FromPrimitive + core::fmt::Debug>(
    result: Result<LogisticFit<T>, StatsError>,
    what: &str,
) -> LogisticFit<T> {
    match result {
        Ok(fit) => fit,
        Err(e) => panic!("{what}: expected a fit, got the typed error {e:?}"),
    }
}

/// Unwraps the error variant, reporting the coefficients if a fit came back instead.
fn expect_error<T: RealField + FromPrimitive + core::fmt::Debug>(
    result: Result<LogisticFit<T>, StatsError>,
    what: &str,
) -> StatsErrorEnum {
    match result {
        Ok(fit) => panic!(
            "{what}: expected a typed error, got a fit with beta {:?}",
            fit.beta
        ),
        Err(StatsError(variant)) => variant,
    }
}

/// The score (gradient of the unpenalised log-likelihood) at `beta`:
/// `g_j = Σ_i x_ij (p_i − y_i)` with `p_i = 1/(1 + exp(−Σ_j x_ij β_j))`.
///
/// This is the first-order condition that *defines* the unpenalised estimate, not the
/// algorithm that produces it: `fit_logistic` reaches `β` by an iteratively reweighted
/// least-squares solve, and this is a plain sum over the rows. The link is written in its
/// naive form on purpose — it is a different computation from the branch-stable `sigmoid`
/// under test, and it is accurate here because these fixtures keep `|η|` below about ten.
fn score_residual<T: RealField>(x: &[Vec<T>], y: &[T], beta: &[T]) -> Vec<T> {
    let mut g = vec![T::zero(); beta.len()];
    for (row, label) in x.iter().zip(y.iter()) {
        let mut eta = T::zero();
        for (entry, coefficient) in row.iter().zip(beta.iter()) {
            eta += *entry * *coefficient;
        }
        let p = T::one() / (T::one() + (-eta).exp());
        let residual = p - *label;
        for (component, entry) in g.iter_mut().zip(row.iter()) {
            *component += *entry * residual;
        }
    }
    g
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Eight observations whose labels overlap in both directions — the label 1 appears at
/// x = −2, left of the label 0 at x = −1, 0 and 1 — so no threshold separates them and the
/// unpenalised maximum-likelihood estimate is finite and unique. The trend is still strongly
/// increasing: the mean x of the ones is 0.875 and of the zeros −0.75.
///
/// Column 0 is an explicit intercept. Column 1 contains an exact zero (row F) and negative
/// entries (row G). Four labels are 1 and four are 0, so the base rate is exactly 1/2.
const OVERLAP_X: [[f64; 2]; 8] = [
    [1.0, -3.0],
    [1.0, -2.0],
    [1.0, -1.0],
    [1.0, 0.0],
    [1.0, 0.5],
    [1.0, 1.0],
    [1.0, 2.0],
    [1.0, 3.0],
];
const OVERLAP_Y: [f64; 8] = [0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0];

/// Four observations separated by the threshold x = 0, so the unpenalised likelihood has no
/// maximum: it increases without bound as the slope grows.
///
/// The design is mirror-symmetric — the map x ↦ −x carries every observation onto one with
/// the opposite label — which pins the intercept exactly (see the symmetry argument in
/// `fit_logistic_on_separable_data_under_a_penalty_is_bounded_and_symmetric`).
const SEPARABLE_X: [[f64; 2]; 4] = [[1.0, -2.0], [1.0, -1.0], [1.0, 1.0], [1.0, 2.0]];
const SEPARABLE_Y: [f64; 4] = [0.0, 0.0, 1.0, 1.0];

// ===========================================================================
// sigmoid
// ===========================================================================

fn check_sigmoid_at_zero<T: RealField + FromPrimitive + core::fmt::Debug>() {
    // Closed form, evaluated by hand: σ(0) = 1/(1 + e^{−0}) = 1/(1 + 1) = 1/2.
    // One half is exactly representable at every binary precision, so this is an exact
    // equality with no tolerance. Row C as well as row F: a numerically stable sigmoid is
    // written as two branches, one for x ≥ 0 and one for x < 0, and zero is the argument
    // where the two branches must agree.
    let got = sigmoid(lift::<T>(0.0));
    assert_eq!(
        got,
        lift::<T>(0.5),
        "sigmoid(0) must be exactly 1/2, got {got:?}"
    );
}

#[test]
fn sigmoid_at_zero_is_exactly_one_half() {
    check_sigmoid_at_zero::<f32>();
    check_sigmoid_at_zero::<f64>();
    check_sigmoid_at_zero::<Float106>();
}

fn check_sigmoid_reflection<T: RealField + FromPrimitive + core::fmt::Debug>(tol: Prec) {
    // Algebraic invariant, not a value: σ(−x) = 1 − σ(x) for every x, because
    // 1/(1 + e^{x}) = e^{−x}/(e^{−x} + 1) = 1 − 1/(1 + e^{−x}).
    // The comparison is absolute rather than relative: at large x the quantity 1 − σ(x) is
    // computed by cancellation against 1.0 and its absolute error is the spacing at 1.0.
    let one = lift::<T>(1.0);
    let slack = lift::<T>(tol.native);
    for &x in &[0.0, 0.25, 0.75, 1.0, 2.5, 6.0, 12.0, 40.0] {
        let positive = sigmoid(lift::<T>(x));
        let negative = sigmoid(lift::<T>(-x));
        assert!(
            close(negative, one - positive, slack),
            "sigmoid(-{x}) = {negative:?} must equal 1 - sigmoid({x}) = {:?}",
            one - positive
        );
    }
}

#[test]
fn sigmoid_is_the_reflection_of_its_complement() {
    check_sigmoid_reflection::<f32>(F32);
    check_sigmoid_reflection::<f64>(F64);
    check_sigmoid_reflection::<Float106>(F106);
}

fn check_sigmoid_closed_forms<T: RealField + FromPrimitive + core::fmt::Debug>(tol: Prec) {
    let slack = lift::<T>(tol.literal);

    // Closed form σ(1) = 1/(1 + e^{−1}).
    //   e      = 2.718281828459045235360287471352...
    //   e^{−1} = 0.367879441171442321595523770161...
    //   1 + e^{−1} = 1.367879441171442321595523770161...
    //   1 / that   = 0.731058578630004879251159241821...
    // The decimal expansion is the standard tabulated value of the logistic function at 1.
    let at_one = sigmoid(lift::<T>(1.0));
    let expected_one = lift::<T>(0.7310585786300049);
    assert!(
        close(at_one, expected_one, slack),
        "sigmoid(1) = {at_one:?} must equal 0.7310585786300049 to {:?}",
        slack
    );

    // Closed form σ(−1) = 1 − σ(1) = 0.268941421369995120748840758178...
    let at_minus_one = sigmoid(lift::<T>(-1.0));
    let expected_minus_one = lift::<T>(0.2689414213699951);
    assert!(
        close(at_minus_one, expected_minus_one, slack),
        "sigmoid(-1) = {at_minus_one:?} must equal 0.2689414213699951 to {:?}",
        slack
    );

    // Closed form at the one argument where the value is rational:
    //   σ(ln 2) = 1/(1 + e^{−ln 2}) = 1/(1 + 1/2) = 1/(3/2) = 2/3 = 0.666666666666...
    // The argument is the published constant ln 2 = 0.693147180559945309417232121458...,
    // taken as `core::f64::consts::LN_2` rather than written out as a literal because
    // clippy rejects a literal approximation of a named constant. The nearest f64 to ln 2
    // sits 2.3e-17 below it, and σ' = σ(1 − σ) = 2/9 there, so that displaces the expected
    // value by only 5.2e-18 — inside every `literal` slack above.
    let at_ln_two = sigmoid(lift::<T>(core::f64::consts::LN_2));
    let two_thirds = lift::<T>(0.6666666666666666);
    assert!(
        close(at_ln_two, two_thirds, slack),
        "sigmoid(ln 2) = {at_ln_two:?} must equal 2/3 to {:?}",
        slack
    );
}

#[test]
fn sigmoid_matches_hand_evaluated_closed_forms() {
    check_sigmoid_closed_forms::<f32>(F32);
    check_sigmoid_closed_forms::<f64>(F64);
    check_sigmoid_closed_forms::<Float106>(F106);
}

fn check_sigmoid_bounded_and_increasing<T: RealField + FromPrimitive + core::fmt::Debug>() {
    // A property over a generated family: 0 < σ(x) < 1 and σ is strictly increasing.
    //
    // The grid stops at ±12 because of f32, not because of the mathematics: the spacing just
    // below 1.0 at f32 is 6.0e-8, and 1 − σ(x) drops below that for x above about 17, where
    // the strict upper bound stops being *representable* rather than stopping being true.
    // At x = 12 the gap is 1 − σ(12) = 6.1e-6, three orders above the f32 spacing.
    let zero = lift::<T>(0.0);
    let one = lift::<T>(1.0);
    let mut previous: Option<T> = None;
    for step in 0..=48 {
        let argument = -12.0 + 0.5 * f64::from(step);
        let value = sigmoid(lift::<T>(argument));
        assert!(
            value > zero,
            "sigmoid({argument}) = {value:?} must be strictly above 0"
        );
        assert!(
            value < one,
            "sigmoid({argument}) = {value:?} must be strictly below 1"
        );
        if let Some(earlier) = previous {
            assert!(
                earlier < value,
                "sigmoid must increase strictly: sigmoid({}) = {earlier:?} is not below sigmoid({argument}) = {value:?}",
                argument - 0.5
            );
        }
        previous = Some(value);
    }
}

#[test]
fn sigmoid_is_bounded_in_the_open_unit_interval_and_strictly_increasing() {
    check_sigmoid_bounded_and_increasing::<f32>();
    check_sigmoid_bounded_and_increasing::<f64>();
    check_sigmoid_bounded_and_increasing::<Float106>();
}

fn check_sigmoid_saturation<T: RealField + FromPrimitive + core::fmt::Debug>(tol: Prec) {
    // Row J. `exp` overflows above about 88.7 at f32 and about 709.8 at f64 and Float106, so
    // exp(800) is +inf at all three. A form that evaluates the positive exponential —
    // e^{x}/(1 + e^{x}), or the wrong branch of a two-branch guard — computes inf/(1 + inf)
    // and returns NaN here. The saturated values are the closed-form limits:
    //   σ(800)  = 1/(1 + e^{−800}), and e^{−800} ≈ 3.6e-348 underflows to 0, so σ(800) = 1.
    //   σ(−800) = 1 − σ(800) = 0.
    let zero = lift::<T>(0.0);
    let one = lift::<T>(1.0);
    let slack = lift::<T>(tol.literal);

    let high = sigmoid(lift::<T>(800.0));
    assert!(
        high.is_finite(),
        "sigmoid(800) must be finite, got {high:?}"
    );
    assert!(high <= one, "sigmoid(800) = {high:?} must not exceed 1");
    assert!(
        close(high, one, slack),
        "sigmoid(800) = {high:?} must saturate at 1"
    );

    let low = sigmoid(lift::<T>(-800.0));
    assert!(low.is_finite(), "sigmoid(-800) must be finite, got {low:?}");
    assert!(low >= zero, "sigmoid(-800) = {low:?} must not fall below 0");
    assert!(
        close(low, zero, slack),
        "sigmoid(-800) = {low:?} must saturate at 0"
    );
}

#[test]
fn sigmoid_saturates_where_exp_overflows() {
    check_sigmoid_saturation::<f32>(F32);
    check_sigmoid_saturation::<f64>(F64);
    check_sigmoid_saturation::<Float106>(F106);
}

fn check_sigmoid_at_the_extreme<T: RealField + FromPrimitive + core::fmt::Debug>(tol: Prec) {
    // Row J at each type's *own* extreme rather than at a shared constant: 1e20 sits within
    // an order of magnitude of nothing in particular at f64, but `tol.huge` is chosen per
    // precision so that the argument is near that precision's overflow reach. The limits are
    // the same closed forms as above: σ(+huge) = 1 and σ(−huge) = 0.
    let zero = lift::<T>(0.0);
    let one = lift::<T>(1.0);
    let slack = lift::<T>(tol.literal);
    let huge = lift::<T>(tol.huge);

    let high = sigmoid(huge);
    assert!(
        high.is_finite(),
        "sigmoid at the precision's extreme must be finite, got {high:?}"
    );
    assert!(
        close(high, one, slack),
        "sigmoid({:?}) = {high:?} must saturate at 1",
        huge
    );

    let low = sigmoid(-huge);
    assert!(
        low.is_finite(),
        "sigmoid at the precision's negative extreme must be finite, got {low:?}"
    );
    assert!(
        close(low, zero, slack),
        "sigmoid({:?}) = {low:?} must saturate at 0",
        -huge
    );
}

#[test]
fn sigmoid_saturates_at_the_precisions_own_extreme() {
    check_sigmoid_at_the_extreme::<f32>(F32);
    check_sigmoid_at_the_extreme::<f64>(F64);
    check_sigmoid_at_the_extreme::<Float106>(F106);
}

fn check_sigmoid_non_finite<T: RealField + FromPrimitive + core::fmt::Debug>() {
    // Row I. The doc does not name the non-finite cases; the reading asserted is the
    // continuous extension of 1/(1 + e^{−x}) to the extended reals:
    //   σ(+∞) = 1/(1 + e^{−∞}) = 1/(1 + 0) = 1, exactly.
    //   σ(−∞) = 1/(1 + ∞)               = 0, exactly.
    //   σ(NaN) is NaN: there is no value to return and the signature has no error channel.
    // `FromPrimitive::from_f64` is a value-preserving cast at all three precisions, so the
    // non-finite fixtures survive the lift.
    let nan = sigmoid(lift::<T>(f64::NAN));
    assert!(
        nan.is_nan(),
        "sigmoid(NaN) must propagate the NaN, got {nan:?}"
    );

    let at_infinity = sigmoid(lift::<T>(f64::INFINITY));
    assert_eq!(
        at_infinity,
        lift::<T>(1.0),
        "sigmoid(+inf) must be exactly 1"
    );

    let at_negative_infinity = sigmoid(lift::<T>(f64::NEG_INFINITY));
    assert_eq!(
        at_negative_infinity,
        lift::<T>(0.0),
        "sigmoid(-inf) must be exactly 0"
    );
}

#[test]
fn sigmoid_maps_non_finite_arguments_to_their_limits() {
    check_sigmoid_non_finite::<f32>();
    check_sigmoid_non_finite::<f64>();
    check_sigmoid_non_finite::<Float106>();
}

// ===========================================================================
// fit_logistic — convergence and the iteration count
// ===========================================================================

fn check_fit_converges<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    let x = design::<T, 2>(&OVERLAP_X);
    let y = lift_array::<T>(&OVERLAP_Y);
    let fit = expect_fit(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(lift::<T>(0.01), 100, lift::<T>(fit_cfg.tolerance)),
        ),
        "well-separated but overlapping data at a small penalty",
    );

    // `LogisticFit::beta` is documented as one coefficient per column of the design, and the
    // design here has two columns. This assertion is what pins the reading that no intercept
    // column is added on the caller's behalf.
    assert_eq!(
        fit.beta.len(),
        2,
        "one coefficient per column of the design"
    );

    for coefficient in &fit.beta {
        assert!(
            coefficient.is_finite(),
            "a converged fit has finite coefficients, got {:?}",
            fit.beta
        );
    }

    // The count is reported on success too. It cannot be zero — the estimate is not the zero
    // vector, so at least one step was taken — and it cannot exceed the cap.
    assert!(
        fit.iterations >= 1,
        "a converged fit reports the steps it took, got 0"
    );
    assert!(
        fit.iterations <= 100,
        "the reported count cannot exceed the cap, got {}",
        fit.iterations
    );

    // Qualitative invariant, not a value: the labels increase with column 1 (the mean x of
    // the ones is 0.875 against −0.75 for the zeros), so the fitted log-odds must increase
    // in it. A sign error anywhere in the residual or the solve flips this.
    assert!(
        fit.beta[1] > T::zero(),
        "the slope must be positive for data whose labels increase with x, got {:?}",
        fit.beta
    );
}

#[test]
fn fit_logistic_converges_on_overlapping_data_and_reports_its_iteration_count() {
    check_fit_converges::<f32>(FIT_F32);
    check_fit_converges::<f64>(FIT_F64);
    check_fit_converges::<Float106>(FIT_F106);
}

fn check_fit_satisfies_the_score_equation<T: RealField + FromPrimitive + core::fmt::Debug>(
    tol: Prec,
    fit_cfg: Fit,
) {
    // At zero penalty the estimate is defined by Xᵀ(p − y) = 0, with no penalty convention
    // to guess at. The fixture is not separable, so that solution exists and is unique.
    let x = design::<T, 2>(&OVERLAP_X);
    let y = lift_array::<T>(&OVERLAP_Y);
    let fit = expect_fit(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(T::zero(), 100, lift::<T>(fit_cfg.tolerance)),
        ),
        "overlapping data at zero penalty",
    );

    // The oracle is pinned first at a point where it does *not* vanish, so that a broken
    // oracle cannot pass the assertion below by returning zero everywhere. At beta = 0 every
    // fitted probability is 1/2, so g_j = sum_i x_ij (1/2 - y_i).
    //   Column 0 is all ones and the labels are four ones and four zeros:
    //     g_0 = 8*(1/2) - sum y_i = 4 - 4 = 0.
    //   Column 1 is (-3, -2, -1, 0, 0.5, 1, 2, 3), with sum x = 0.5 and
    //   sum x*y = -2 + 0.5 + 2 + 3 = 3.5:
    //     g_1 = (1/2)(0.5) - 3.5 = 0.25 - 3.5 = -3.25.
    let at_origin = score_residual(&x, &y, &[T::zero(), T::zero()]);
    let exact = lift::<T>(tol.native);
    assert!(
        close(at_origin[0], lift::<T>(0.0), exact),
        "score component 0 at the origin is 0 by hand arithmetic, got {:?}",
        at_origin[0]
    );
    assert!(
        close(at_origin[1], lift::<T>(-3.25), exact),
        "score component 1 at the origin is -3.25 by hand arithmetic, got {:?}",
        at_origin[1]
    );

    // And now the estimate itself: the same oracle must vanish there.
    let g = score_residual(&x, &y, &fit.beta);
    let slack = lift::<T>(fit_cfg.converged);
    for (index, component) in g.iter().enumerate() {
        assert!(
            close(*component, T::zero(), slack),
            "the unpenalised estimate must solve its own score equation: component {index} is {component:?}, beta {:?}",
            fit.beta
        );
    }
}

#[test]
fn fit_logistic_solves_the_unpenalised_score_equation() {
    check_fit_satisfies_the_score_equation::<f32>(F32, FIT_F32);
    check_fit_satisfies_the_score_equation::<f64>(F64, FIT_F64);
    check_fit_satisfies_the_score_equation::<Float106>(F106, FIT_F106);
}

fn check_iterations_monotone_in_tolerance<T: RealField + FromPrimitive + core::fmt::Debug>() {
    // Invariant over a generated family. The sequence of iterates depends on the data and
    // the penalty, never on the tolerance; the tolerance only decides where the sequence is
    // cut. A harder stopping test therefore cannot be met sooner, whichever quantity the
    // stopping test measures. This is what pins `iterations` to the stopping test rather
    // than to a constant or to the cap.
    let x = design::<T, 2>(&OVERLAP_X);
    let y = lift_array::<T>(&OVERLAP_Y);
    let penalty = lift::<T>(0.01);
    let mut previous = 0usize;
    for &tolerance in &[1e-1, 1e-2, 1e-3, 1e-4] {
        let fit = expect_fit(
            fit_logistic(
                &x,
                &y,
                &LogisticConfig::new(penalty, 200, lift::<T>(tolerance)),
            ),
            "overlapping data at a reachable tolerance",
        );
        assert!(
            fit.iterations >= previous,
            "tightening the tolerance to {tolerance} cannot need fewer iterations: {} after {previous}",
            fit.iterations
        );
        previous = fit.iterations;
    }
}

#[test]
fn fit_logistic_iteration_count_grows_as_the_tolerance_tightens() {
    check_iterations_monotone_in_tolerance::<f32>();
    check_iterations_monotone_in_tolerance::<f64>();
    check_iterations_monotone_in_tolerance::<Float106>();
}

fn check_non_convergence<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    // Row E, both sides of the iteration cap, on one data set at one tolerance.
    //
    // Below the cap: from β = 0 the first two Newton steps are of order 1 and 0.1, nowhere
    // near `fit_tolerance` (1e-5 at f32, 1e-12 at f64, 1e-20 at Float106), so a cap of one or
    // two cannot converge. The reading asserted for the reported count is that reaching the
    // cap means the cap's worth of iterations was performed and none met the stopping test.
    let x = design::<T, 2>(&OVERLAP_X);
    let y = lift_array::<T>(&OVERLAP_Y);
    let penalty = lift::<T>(0.01);
    let tolerance = lift::<T>(fit_cfg.tolerance);

    for &cap in &[1usize, 2] {
        let variant = expect_error(
            fit_logistic(&x, &y, &LogisticConfig::new(penalty, cap, tolerance)),
            "a cap below what the tolerance needs",
        );
        match variant {
            StatsErrorEnum::NotConverged { iterations, .. } => {
                assert_eq!(
                    iterations, cap,
                    "the reported count is the cap that was reached"
                );
            }
            other => panic!("a fit stopped by its cap must report NotConverged, got {other:?}"),
        }
    }

    // Above the cap: the same data and the same tolerance, with room to converge.
    let fit = expect_fit(
        fit_logistic(&x, &y, &LogisticConfig::new(penalty, 200, tolerance)),
        "the same fit given room to converge",
    );
    // Both smaller caps failed at this tolerance, so a converged fit must have needed more
    // than two iterations. This is entailed by the two errors above, not assumed.
    assert!(
        fit.iterations >= 3,
        "caps of 1 and 2 did not converge at this tolerance, so the count must exceed 2, got {}",
        fit.iterations
    );
    assert!(
        fit.iterations <= 200,
        "the reported count cannot exceed the cap, got {}",
        fit.iterations
    );
}

#[test]
fn fit_logistic_reports_non_convergence_with_the_iteration_count() {
    check_non_convergence::<f32>(FIT_F32);
    check_non_convergence::<f64>(FIT_F64);
    check_non_convergence::<Float106>(FIT_F106);
}

fn check_zero_iteration_cap<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    // Row F on the cap itself. A cap of zero performs no iteration, so no stopping test can
    // be met and no coefficient has been estimated. The reading asserted: the same typed
    // non-convergence error, carrying the count actually performed, which is zero. The doc
    // gives the count so a caller can tell a cap that is too low from a problem that does not
    // converge, and a cap of zero is the extreme of the first.
    let x = design::<T, 2>(&OVERLAP_X);
    let y = lift_array::<T>(&OVERLAP_Y);
    let variant = expect_error(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(lift::<T>(0.01), 0, lift::<T>(fit_cfg.tolerance)),
        ),
        "a cap of zero iterations",
    );
    match variant {
        StatsErrorEnum::NotConverged { iterations, .. } => {
            assert_eq!(iterations, 0, "a cap of zero performs no iteration");
        }
        other => panic!("a cap of zero must report NotConverged, got {other:?}"),
    }
}

#[test]
fn fit_logistic_with_a_zero_iteration_cap_reports_zero_iterations() {
    check_zero_iteration_cap::<f32>(FIT_F32);
    check_zero_iteration_cap::<f64>(FIT_F64);
    check_zero_iteration_cap::<Float106>(FIT_F106);
}

// ===========================================================================
// fit_logistic — separable data and the penalty
// ===========================================================================

fn check_separable_at_zero_penalty<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    // The unpenalised likelihood on separable data has no maximum: scaling the slope up
    // strictly increases it, without bound. The iterate therefore diverges instead of
    // settling, and the last iterate is an arbitrarily large number, not an estimate — so it
    // must not come back as a success.
    let x = design::<T, 2>(&SEPARABLE_X);
    let y = lift_array::<T>(&SEPARABLE_Y);
    let variant = expect_error(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(T::zero(), 50, lift::<T>(fit_cfg.tolerance)),
        ),
        "perfectly separable data at zero penalty",
    );
    match variant {
        StatsErrorEnum::NotConverged { iterations, .. } => {
            // The count is bounded rather than pinned to the cap: an implementation may stop
            // early once the weights underflow and the Hessian is no longer usable, and that
            // is still a report of non-convergence at the iteration it reached.
            assert!(
                iterations <= 50,
                "the reported count cannot exceed the cap, got {iterations}"
            );
        }
        other => panic!("separable data at zero penalty must report NotConverged, got {other:?}"),
    }
}

#[test]
fn fit_logistic_on_separable_data_at_zero_penalty_reports_non_convergence() {
    check_separable_at_zero_penalty::<f32>(FIT_F32);
    check_separable_at_zero_penalty::<f64>(FIT_F64);
    check_separable_at_zero_penalty::<Float106>(FIT_F106);
}

fn check_separable_under_penalty<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    let x = design::<T, 2>(&SEPARABLE_X);
    let y = lift_array::<T>(&SEPARABLE_Y);
    let fit = expect_fit(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(lift::<T>(1.0), 200, lift::<T>(fit_cfg.tolerance)),
        ),
        "perfectly separable data at a penalty of one",
    );
    assert_eq!(
        fit.beta.len(),
        2,
        "one coefficient per column of the design"
    );
    for coefficient in &fit.beta {
        assert!(
            coefficient.is_finite(),
            "a penalised fit has finite coefficients, got {:?}",
            fit.beta
        );
    }

    // The penalty bounds the estimate, and the bound is a closed form. The penalised
    // objective is at least the penalty term alone, and its value at the optimum is at most
    // its value at β = 0, which is the unpenalised loss there: every fitted probability is
    // 1/2, so the loss is n·ln 2 = 4 × 0.693147180559945 = 2.7725887222397812.
    //   (λ/2)‖β̂‖² ≤ 2.7725887  ⇒  ‖β̂‖ ≤ sqrt(2 × 2.7725887 / 1) = sqrt(5.5451774) = 2.3547
    // under the (λ/2)‖β‖² convention, and less under λ‖β‖². The assertion uses 10, four
    // times the loosest of those, because the penalty convention is not documented — while
    // still being a real bound: without the penalty this same fit diverges.
    let norm = l2_norm(&fit.beta);
    assert!(
        norm <= lift::<T>(10.0),
        "a penalty of one bounds the coefficient norm well below 10, got {norm:?} for {:?}",
        fit.beta
    );
    assert!(
        fit.beta[1] > T::zero(),
        "the slope must be positive, got {:?}",
        fit.beta
    );

    // Exact invariant from the design's mirror symmetry. Writing the loss for one
    // observation as ℓ(β₀, β₁; x, y) = −y·η + log(1 + e^{η}) with η = β₀ + β₁x, and using
    // log(1 + e^{−u}) = log(1 + e^{u}) − u, one gets
    //   ℓ(−β₀, β₁; x, y) = ℓ(β₀, β₁; −x, 1 − y).
    // The map (x, y) ↦ (−x, 1 − y) permutes this design exactly — (−2,0)↔(2,1) and
    // (−1,0)↔(1,1) — and ‖β‖² is even in β₀, so the whole penalised objective is even in β₀.
    // A strictly convex objective has one minimiser, which must then satisfy β₀ = −β₀, so
    // β₀ = 0. The slack is the `converged` slack: the fit reaches the optimum only as
    // closely as its tolerance allows.
    assert!(
        close(fit.beta[0], T::zero(), lift::<T>(fit_cfg.converged)),
        "the mirror-symmetric design forces a zero intercept, got {:?}",
        fit.beta
    );
}

#[test]
fn fit_logistic_on_separable_data_under_a_penalty_is_bounded_and_symmetric() {
    check_separable_under_penalty::<f32>(FIT_F32);
    check_separable_under_penalty::<f64>(FIT_F64);
    check_separable_under_penalty::<Float106>(FIT_F106);
}

fn check_penalty_shrinks_the_norm<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    // A property over a generated family: the coefficient norm falls as the penalty rises.
    // No value is predicted, only the ordering. The base rate of this fixture is exactly
    // 1/2, so the intercept tends to zero as the penalty grows whether or not the intercept
    // column is penalised, and the ordering does not depend on that convention.
    let x = design::<T, 2>(&OVERLAP_X);
    let y = lift_array::<T>(&OVERLAP_Y);
    let mut previous: Option<T> = None;
    for &penalty in &[0.1, 0.5, 1.0, 2.0, 5.0, 10.0] {
        let fit = expect_fit(
            fit_logistic(
                &x,
                &y,
                &LogisticConfig::new(lift::<T>(penalty), 200, lift::<T>(fit_cfg.tolerance)),
            ),
            "overlapping data at an increasing penalty",
        );
        let norm = l2_norm(&fit.beta);
        if let Some(earlier) = previous {
            assert!(
                norm < earlier,
                "raising the penalty to {penalty} must shrink the coefficient norm: {norm:?} is not below {earlier:?}"
            );
        }
        previous = Some(norm);
    }
}

#[test]
fn fit_logistic_increasing_the_penalty_shrinks_the_coefficient_norm() {
    check_penalty_shrinks_the_norm::<f32>(FIT_F32);
    check_penalty_shrinks_the_norm::<f64>(FIT_F64);
    check_penalty_shrinks_the_norm::<Float106>(FIT_F106);
}

// ===========================================================================
// fit_logistic — refused inputs
// ===========================================================================

fn check_empty_design<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    // Row A, in both shapes an empty design takes.
    let penalty = lift::<T>(1.0);
    let tolerance = lift::<T>(fit_cfg.tolerance);

    let no_rows: Vec<Vec<T>> = Vec::new();
    let no_labels: Vec<T> = Vec::new();
    let variant = expect_error(
        fit_logistic(
            &no_rows,
            &no_labels,
            &LogisticConfig::new(penalty, 100, tolerance),
        ),
        "a design with no observations",
    );
    match variant {
        StatsErrorEnum::EmptyInput(_) => {}
        other => panic!("no observations must be EmptyInput, got {other:?}"),
    }

    // Rows present, but no columns: there is no coefficient to estimate. The doc does not
    // name this shape, so the assertion accepts either of the two variants that describe it
    // — the input is empty in the dimension that matters, or the shape is wrong — and
    // refuses everything else, including a fit with an empty coefficient vector.
    let no_columns: Vec<Vec<T>> = vec![Vec::new(), Vec::new()];
    let two_labels = lift_array::<T>(&[0.0, 1.0]);
    let variant = expect_error(
        fit_logistic(
            &no_columns,
            &two_labels,
            &LogisticConfig::new(penalty, 100, tolerance),
        ),
        "a design with no columns",
    );
    match variant {
        StatsErrorEnum::EmptyInput(_) | StatsErrorEnum::DimensionMismatch(_) => {}
        other => panic!("a design with no columns must be refused as an input, got {other:?}"),
    }
}

#[test]
fn fit_logistic_refuses_an_empty_design() {
    check_empty_design::<f32>(FIT_F32);
    check_empty_design::<f64>(FIT_F64);
    check_empty_design::<Float106>(FIT_F106);
}

fn check_length_mismatch<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    // Three rows, two labels.
    let x = design::<T, 2>(&[[1.0, -1.0], [1.0, 0.0], [1.0, 1.0]]);
    let y = lift_array::<T>(&[0.0, 1.0]);
    let variant = expect_error(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(lift::<T>(1.0), 100, lift::<T>(fit_cfg.tolerance)),
        ),
        "three rows against two labels",
    );
    match variant {
        StatsErrorEnum::DimensionMismatch(_) => {}
        other => panic!(
            "a row count that disagrees with the label count must be DimensionMismatch, got {other:?}"
        ),
    }
}

#[test]
fn fit_logistic_refuses_a_design_and_label_length_mismatch() {
    check_length_mismatch::<f32>(FIT_F32);
    check_length_mismatch::<f64>(FIT_F64);
    check_length_mismatch::<Float106>(FIT_F106);
}

fn check_ragged_rows<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    // The third row carries a column the others do not, so the design is not a matrix.
    let x: Vec<Vec<T>> = vec![
        lift_array::<T>(&[1.0, -1.0]),
        lift_array::<T>(&[1.0, 0.0]),
        lift_array::<T>(&[1.0, 1.0, 2.0]),
    ];
    let y = lift_array::<T>(&[0.0, 1.0, 1.0]);
    let variant = expect_error(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(lift::<T>(1.0), 100, lift::<T>(fit_cfg.tolerance)),
        ),
        "a ragged design",
    );
    match variant {
        StatsErrorEnum::DimensionMismatch(_) => {}
        other => panic!("rows of differing length must be DimensionMismatch, got {other:?}"),
    }
}

#[test]
fn fit_logistic_refuses_ragged_rows() {
    check_ragged_rows::<f32>(FIT_F32);
    check_ragged_rows::<f64>(FIT_F64);
    check_ragged_rows::<Float106>(FIT_F106);
}

fn check_label_domain<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    // Row H, both sides of the label domain.
    //
    // Inside, at the exact boundary: labels of exactly 0 and exactly 1 are the ordinary
    // input and are fitted, not refused.
    let x = design::<T, 2>(&OVERLAP_X);
    let y = lift_array::<T>(&OVERLAP_Y);
    let fit = expect_fit(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(lift::<T>(1.0), 200, lift::<T>(fit_cfg.tolerance)),
        ),
        "labels at exactly 0 and exactly 1",
    );
    assert_eq!(
        fit.beta.len(),
        2,
        "one coefficient per column of the design"
    );

    // Outside: a label that is not a probability. The reading asserted is that both a
    // negative label and one above one are refused under `NegativeProbability`, the crate's
    // one variant for an input that is not a valid probability — the enum carries no
    // label-specific variant, and the doc groups this with the inputs "the mathematics does
    // not admit".
    let mut negative = y.clone();
    negative[0] = lift::<T>(-1.0);
    let variant = expect_error(
        fit_logistic(
            &x,
            &negative,
            &LogisticConfig::new(lift::<T>(1.0), 100, lift::<T>(fit_cfg.tolerance)),
        ),
        "a label of -1",
    );
    match variant {
        StatsErrorEnum::NegativeProbability(_) => {}
        other => panic!("a negative label must be NegativeProbability, got {other:?}"),
    }

    let mut above_one = y.clone();
    above_one[3] = lift::<T>(2.0);
    let variant = expect_error(
        fit_logistic(
            &x,
            &above_one,
            &LogisticConfig::new(lift::<T>(1.0), 100, lift::<T>(fit_cfg.tolerance)),
        ),
        "a label of 2",
    );
    match variant {
        StatsErrorEnum::NegativeProbability(_) => {}
        other => panic!("a label above one must be refused as a probability, got {other:?}"),
    }
}

#[test]
fn fit_logistic_refuses_a_label_outside_the_unit_interval() {
    check_label_domain::<f32>(FIT_F32);
    check_label_domain::<f64>(FIT_F64);
    check_label_domain::<Float106>(FIT_F106);
}

fn check_non_finite_input<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    // Row I. A NaN entry has no meaning in a design, and an infinite label has none either;
    // both reach a comparison in the stopping test, where they would silently decide
    // convergence.
    let y = lift_array::<T>(&OVERLAP_Y);
    let mut x = design::<T, 2>(&OVERLAP_X);
    x[2][1] = lift::<T>(f64::NAN);
    let variant = expect_error(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(lift::<T>(1.0), 100, lift::<T>(fit_cfg.tolerance)),
        ),
        "a NaN design entry",
    );
    match variant {
        StatsErrorEnum::NonFiniteInput(_) => {}
        other => panic!("a NaN design entry must be NonFiniteInput, got {other:?}"),
    }

    let x = design::<T, 2>(&OVERLAP_X);
    let mut infinite = y.clone();
    infinite[0] = lift::<T>(f64::INFINITY);
    let variant = expect_error(
        fit_logistic(
            &x,
            &infinite,
            &LogisticConfig::new(lift::<T>(1.0), 100, lift::<T>(fit_cfg.tolerance)),
        ),
        "an infinite label",
    );
    match variant {
        StatsErrorEnum::NonFiniteInput(_) => {}
        other => panic!("an infinite label must be NonFiniteInput, got {other:?}"),
    }
}

#[test]
fn fit_logistic_refuses_a_non_finite_design_or_label() {
    check_non_finite_input::<f32>(FIT_F32);
    check_non_finite_input::<f64>(FIT_F64);
    check_non_finite_input::<Float106>(FIT_F106);
}

fn check_single_observation<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    // Rows B and D together: one observation and one column, so n = 1, p = 1, and every
    // index expression over the design collapses onto a single entry.
    let x: Vec<Vec<T>> = vec![lift_array::<T>(&[1.0])];
    let y = lift_array::<T>(&[1.0]);
    let fit = expect_fit(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(lift::<T>(1.0), 100, lift::<T>(fit_cfg.tolerance)),
        ),
        "a single observation at a penalty of one",
    );
    assert_eq!(
        fit.beta.len(),
        1,
        "one coefficient per column of the design"
    );
    assert!(
        fit.beta[0].is_finite(),
        "the single coefficient must be finite, got {:?}",
        fit.beta
    );

    // The objective is J(β) = log(1 + e^{−β}) + (λ/2)β² for the one observation (x = 1,
    // y = 1). Its derivative at zero is −σ(0) = −1/2 for any λ, so the minimiser lies
    // strictly above zero whatever positive multiple of ‖β‖² the penalty convention uses.
    assert!(
        fit.beta[0] > T::zero(),
        "a single observation labelled 1 pulls the coefficient above zero, got {:?}",
        fit.beta
    );

    // And it is bounded: J(β̂) ≤ J(0) = log 2 = 0.6931471805599453, so under the loosest
    // convention (λ/2)β̂² ≤ 0.6931 gives |β̂| ≤ sqrt(2 × 0.6931) = 1.1774. The assertion
    // uses 5, four times that, for the same reason as the separable case.
    assert!(
        fit.beta[0] <= lift::<T>(5.0),
        "a penalty of one bounds the single coefficient well below 5, got {:?}",
        fit.beta
    );
}

#[test]
fn fit_logistic_fits_a_single_observation_under_a_penalty() {
    check_single_observation::<f32>(FIT_F32);
    check_single_observation::<f64>(FIT_F64);
    check_single_observation::<Float106>(FIT_F106);
}

fn check_rank_deficient<T: RealField + FromPrimitive + core::fmt::Debug>(fit_cfg: Fit) {
    // Rows C and D. The two columns are identical, so the design has rank one with two
    // columns: at zero penalty the objective is flat along β₀ − β₁ and the estimate is not
    // unique. Every entry of the 2×2 Hessian is the same number, which is also the case
    // where an index expression over it degenerates.
    //
    // The labels are chosen so the design is *not* separable — x = 1 carries both a 0 and a
    // 1 — so rank deficiency is the only defect present.
    let x = design::<T, 2>(&[[1.0, 1.0], [1.0, 1.0], [2.0, 2.0], [3.0, 3.0]]);
    let y = lift_array::<T>(&[0.0, 1.0, 0.0, 1.0]);
    let variant = expect_error(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(T::zero(), 100, lift::<T>(fit_cfg.tolerance)),
        ),
        "a design with two identical columns at zero penalty",
    );
    match variant {
        // Either report is defensible for a singular Hessian — the design has no unique
        // solution at the penalty given, or the iteration could not settle on one. What the
        // test pins is that neither one of the infinitely many solutions comes back as a fit.
        StatsErrorEnum::RankDeficient(_) | StatsErrorEnum::NotConverged { .. } => {}
        other => panic!("a rank-deficient design at zero penalty must be refused, got {other:?}"),
    }
}

#[test]
fn fit_logistic_refuses_a_rank_deficient_design_at_zero_penalty() {
    check_rank_deficient::<f32>(FIT_F32);
    check_rank_deficient::<f64>(FIT_F64);
    check_rank_deficient::<Float106>(FIT_F106);
}

fn check_overflow_reach<T: RealField + FromPrimitive + core::fmt::Debug>(tol: Prec, fit_cfg: Fit) {
    // Row J for the fit, at each precision's own reach rather than at a shared constant.
    // Every entry is finite, but the normal equations square them and `huge`² is past the
    // maximum of the type in use (3.4e38 at f32, 1.8e308 at f64 and Float106), so the
    // Hessian entries are not representable.
    //
    // Refusing the design is coverage; so is returning a finite fit. What is not allowed is
    // a success carrying a coefficient that is not a number — the failure mode this whole
    // family exists to remove.
    let huge = tol.huge;
    let x = design::<T, 2>(&[
        [1.0, -huge],
        [1.0, -huge / 2.0],
        [1.0, huge / 2.0],
        [1.0, huge],
    ]);
    let y = lift_array::<T>(&[0.0, 0.0, 1.0, 1.0]);
    match fit_logistic(
        &x,
        &y,
        &LogisticConfig::new(lift::<T>(1.0), 100, lift::<T>(fit_cfg.tolerance)),
    ) {
        Ok(fit) => {
            for coefficient in &fit.beta {
                assert!(
                    coefficient.is_finite(),
                    "a design at the precision's overflow reach must never be returned as a fit with a non-finite coefficient, got {:?}",
                    fit.beta
                );
            }
        }
        Err(_) => {
            // A typed refusal is the other acceptable outcome.
        }
    }
}

#[test]
fn fit_logistic_never_returns_a_non_finite_coefficient() {
    check_overflow_reach::<f32>(F32, FIT_F32);
    check_overflow_reach::<f64>(F64, FIT_F64);
    check_overflow_reach::<Float106>(F106, FIT_F106);
}

// ---------------------------------------------------------------------------------------------
// Each half of the penalty-and-tolerance guard
// ---------------------------------------------------------------------------------------------
//
// The guard is `penalty non-finite || tolerance non-finite`. With both non-finite, `&&` behaves
// the same and the mutant survives; these two cases hold exactly one half each.

#[test]
fn fit_logistic_refuses_a_non_finite_penalty_with_a_finite_tolerance() {
    let x: Vec<Vec<f64>> = vec![vec![1.0, 0.0], vec![1.0, 1.0], vec![1.0, 2.0]];
    let y = vec![0.0, 1.0, 1.0];
    let variant = expect_error(
        fit_logistic(&x, &y, &LogisticConfig::new(f64::NAN, 100, 1e-9)),
        "a NaN penalty with a usable tolerance",
    );
    match variant {
        StatsErrorEnum::NonFiniteInput(_) => {}
        other => panic!("a non-finite penalty must be NonFiniteInput, got {other:?}"),
    }
}

#[test]
fn fit_logistic_refuses_a_non_finite_tolerance_with_a_finite_penalty() {
    let x: Vec<Vec<f64>> = vec![vec![1.0, 0.0], vec![1.0, 1.0], vec![1.0, 2.0]];
    let y = vec![0.0, 1.0, 1.0];
    let variant = expect_error(
        fit_logistic(&x, &y, &LogisticConfig::new(1.0, 100, f64::INFINITY)),
        "an infinite tolerance with a usable penalty",
    );
    match variant {
        StatsErrorEnum::NonFiniteInput(_) => {}
        other => panic!("a non-finite tolerance must be NonFiniteInput, got {other:?}"),
    }
}

/// An exempt intercept preserves the base rate; a penalised one does not.
///
/// On an intercept-only design the logistic MLE is the sample proportion exactly: with
/// `y = (1, 1, 1, 0)` the fitted probability is `3/4`, whatever the penalty, *provided the penalty
/// does not reach the intercept*. Penalising it shrinks the log-odds toward zero, which is the
/// probability toward `1/2`, so the same fit comes back strictly below `3/4` and the base rate is
/// lost. That is the whole content of the convention, on the smallest design that shows it.
///
/// `sigmoid(β₀)` is the fitted probability here because the design is the single ones-column.
fn check_exempt_intercept_keeps_the_base_rate<T: RealField + FromPrimitive + core::fmt::Debug>(
    fit_cfg: Fit,
) {
    let x = design::<T, 1>(&[[1.0], [1.0], [1.0], [1.0]]);
    let y = lift_array::<T>(&[1.0, 1.0, 1.0, 0.0]);
    let penalty = lift::<T>(1.0);
    let tolerance = lift::<T>(fit_cfg.tolerance);

    let exempt = expect_fit(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(penalty, 200, tolerance)
                .with_penalisation(Penalisation::Excluding(0)),
        ),
        "an intercept-only design with the intercept exempt",
    );
    let pi_exempt = sigmoid(exempt.beta[0]);
    let three_quarters = lift::<T>(0.75);
    assert!(
        close(pi_exempt, three_quarters, lift::<T>(fit_cfg.converged)),
        "an exempt intercept fits the sample proportion 3/4 exactly, got {pi_exempt:?}"
    );

    let penalised = expect_fit(
        fit_logistic(&x, &y, &LogisticConfig::new(penalty, 200, tolerance)),
        "the same design with the intercept penalised",
    );
    let pi_penalised = sigmoid(penalised.beta[0]);
    let half = lift::<T>(0.5);
    assert!(
        pi_penalised < pi_exempt && pi_penalised > half,
        "a penalised intercept is pulled from the base rate toward one half: \
         exempt {pi_exempt:?}, penalised {pi_penalised:?}"
    );
}

#[test]
fn test_fit_logistic_exempt_intercept_keeps_the_base_rate() {
    check_exempt_intercept_keeps_the_base_rate::<f64>(FIT_F64);
    check_exempt_intercept_keeps_the_base_rate::<Float106>(FIT_F106);
}

/// Exempting a column the design does not have is refused, as it is for ridge.
#[test]
fn test_fit_logistic_exempt_column_outside_the_design_is_refused() {
    let x = design::<f64, 2>(&[[1.0, 2.0], [1.0, 3.0], [1.0, 4.0]]);
    let y = lift_array::<f64>(&[1.0, 0.0, 1.0]);
    let variant = expect_error(
        fit_logistic(
            &x,
            &y,
            &LogisticConfig::new(1.0, 100, 1e-10).with_penalisation(Penalisation::Excluding(2)),
        ),
        "an exempt column past the design width",
    );
    assert!(
        matches!(variant, StatsErrorEnum::DimensionMismatch(_)),
        "an exempt column past the design width is a shape error, got {variant:?}"
    );
}
