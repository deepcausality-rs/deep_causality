/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `Real` analytic-scalar trait and its relationship to `RealField`.
//!
//! The analytic method surface itself (sqrt/exp/ln/sin/…) is exercised, per type,
//! in `field_real_f32_tests`, `field_real_f64_tests`, and the `float_double` suite
//! (all now through `Real`, which owns those methods). This file covers the trait
//! relationship the refactor introduces: `RealField: Real + Field`, that every
//! concrete real scalar is a `Real`, and that a `Real` bound accepts any `RealField`.

use deep_causality_algebra::{Real, RealField};
use deep_causality_num::Float106;

/// A generic that needs only the analytic surface — bounded on `Real`, not `RealField`.
fn analytic_sqrt<T: Real>(x: T) -> T {
    x.sqrt()
}

/// `RealField` implies `Real`: a `RealField`-bounded value is usable where `Real` is required.
fn through_realfield<T: RealField>(x: T) -> T {
    analytic_sqrt(x)
}

#[test]
fn realfield_implies_real_f64() {
    assert_eq!(through_realfield(9.0_f64), 3.0);
}

#[test]
fn realfield_implies_real_f32() {
    assert_eq!(through_realfield(16.0_f32), 4.0);
}

#[test]
fn real_bound_accepts_concrete_scalars() {
    assert_eq!(analytic_sqrt(25.0_f64), 5.0);
    assert_eq!(analytic_sqrt(49.0_f32), 7.0);
}

#[test]
fn real_surface_constants_and_functions_f64() {
    assert_eq!(<f64 as Real>::pi(), core::f64::consts::PI);
    assert_eq!(<f64 as Real>::e(), core::f64::consts::E);
    assert_eq!(<f64 as Real>::sqrt(4.0), 2.0);
    assert!((<f64 as Real>::exp(1.0) - core::f64::consts::E).abs() < 1e-12);
    assert!((<f64 as Real>::ln(core::f64::consts::E) - 1.0).abs() < 1e-12);
}

#[test]
fn real_surface_constants_and_functions_f32() {
    assert_eq!(<f32 as Real>::pi(), core::f32::consts::PI);
    assert_eq!(<f32 as Real>::sqrt(9.0), 3.0);
}

#[test]
fn concrete_scalars_are_real() {
    fn assert_real<T: Real>() {}
    assert_real::<f32>();
    assert_real::<f64>();
    assert_real::<Float106>();
}

#[test]
fn concrete_scalars_are_realfield() {
    fn assert_real_field<T: RealField>() {}
    assert_real_field::<f32>();
    assert_real_field::<f64>();
    assert_real_field::<Float106>();
}

#[test]
fn float106_real_surface() {
    let four = Float106::from(4.0);
    let two = Float106::from(2.0);
    let diff = <Float106 as Real>::sqrt(four) - two;
    assert!(diff.abs() < Float106::from(1e-12));
}

// --- Real::cbrt ---------------------------------------------------------------------------
//
// Expectations are hand-evaluated exact cubes written as literals, the algebraic invariant
// `cbrt(x)³ = x`, or `powf(1/3)` — a different code path, and only where it agrees. Nothing
// here compares `cbrt` against a retyping of its own implementation.

/// Cube roots that are exact in binary floating point: the radicand is a power of two whose
/// exponent is divisible by three, so both it and its root are representable without rounding.
const EXACT_CUBES_F64: [(f64, f64); 7] = [
    (1.0, 1.0),
    (8.0, 2.0),
    (64.0, 4.0),
    (512.0, 8.0),
    (0.125, 0.5),
    (0.001953125, 0.125),   // 2^-9 -> 2^-3
    (1073741824.0, 1024.0), // 2^30 -> 2^10
];

#[test]
fn cbrt_exact_cubes_f64() {
    for (x, want) in EXACT_CUBES_F64 {
        assert_eq!(Real::cbrt(x), want, "cbrt({x})");
        assert_eq!(Real::cbrt(-x), -want, "cbrt({})", -x);
    }
}

#[test]
fn cbrt_exact_cubes_f32() {
    for (x, want) in EXACT_CUBES_F64 {
        let (x, want) = (x as f32, want as f32);
        assert_eq!(Real::cbrt(x), want, "cbrt({x})");
        assert_eq!(Real::cbrt(-x), -want, "cbrt({})", -x);
    }
}

#[test]
fn cbrt_exact_cubes_float106() {
    for (x, want) in EXACT_CUBES_F64 {
        let got = Real::cbrt(Float106::from(x));
        assert!(
            (got - Float106::from(want)).abs() < Float106::from(1e-30),
            "cbrt({x}) at Float106"
        );
    }
}

/// The decisive case: a negative radicand has a real cube root, and `powf(1/3)` cannot produce
/// it. This is the defect in `signed_cbrt` that motivates putting `cbrt` on the trait.
#[test]
fn cbrt_of_a_negative_is_real_where_powf_is_nan() {
    assert_eq!(Real::cbrt(-27.0_f64), -3.0);
    assert!(Real::powf(-27.0_f64, 1.0 / 3.0).is_nan());

    assert_eq!(Real::cbrt(-27.0_f32), -3.0);
    assert!(Real::powf(-27.0_f32, 1.0 / 3.0).is_nan());

    let neg = Float106::from(-27.0);
    assert!((Real::cbrt(neg) - Float106::from(-3.0)).abs() < Float106::from(1e-30));
    assert!(Real::powf(neg, Float106::from(1.0) / Float106::from(3.0)).is_nan());
}

/// `cbrt(x)³ = x` holds for every real x. An invariant, not a restatement of the algorithm.
#[test]
fn cbrt_cubed_recovers_the_argument_f64() {
    for x in [
        -1e6, -1234.5678, -2.0, -1.0, -0.5, -1e-6, 1e-6, 0.5, 1.0, 2.0, 1234.5678, 1e6,
    ] {
        let r = Real::cbrt(x);
        let back = r * r * r;
        assert!(
            (back - x).abs() <= 1e-9 * x.abs(),
            "cbrt({x})³ = {back}, want {x}"
        );
    }
}

#[test]
fn cbrt_cubed_recovers_the_argument_f32() {
    for x in [-1234.5_f32, -2.0, -0.5, 0.5, 2.0, 1234.5] {
        let r = Real::cbrt(x);
        let back = r * r * r;
        assert!(
            (back - x).abs() <= 1e-4 * x.abs(),
            "cbrt({x})³ = {back}, want {x}"
        );
    }
}

#[test]
fn cbrt_cubed_recovers_the_argument_float106() {
    for x in [-1234.5678_f64, -2.0, -0.5, 0.5, 2.0, 1234.5678] {
        let v = Float106::from(x);
        let r = Real::cbrt(v);
        let back = r * r * r;
        assert!(
            (back - v).abs() <= Float106::from(1e-30) * v.abs(),
            "cbrt({x})³ at Float106"
        );
    }
}

/// On positive arguments `powf(1/3)` is a genuinely different code path and must agree.
#[test]
fn cbrt_agrees_with_powf_on_positives() {
    for x in [1e-6_f64, 0.5, 1.0, 2.0, 7.0, 1234.5678, 1e6] {
        let a = Real::cbrt(x);
        let b = Real::powf(x, 1.0 / 3.0);
        assert!(
            (a - b).abs() <= 1e-12 * a.abs(),
            "cbrt({x}) vs powf: {a} vs {b}"
        );
    }
}

#[test]
fn cbrt_of_zero_is_zero_and_keeps_the_sign() {
    assert_eq!(Real::cbrt(0.0_f64), 0.0);
    assert_eq!(Real::cbrt(-0.0_f64), 0.0);
    // The sign of zero survives, as it does for the primitive operation.
    assert!(Real::cbrt(-0.0_f64).is_sign_negative());
    assert!(Real::cbrt(0.0_f64).is_sign_positive());

    assert_eq!(Real::cbrt(0.0_f32), 0.0);
    assert_eq!(Real::cbrt(-0.0_f32), 0.0);
    assert!(Real::cbrt(Float106::from(0.0)).abs() < Float106::from(1e-30));
}

#[test]
fn cbrt_of_non_finite_arguments() {
    assert!(Real::cbrt(f64::INFINITY).is_infinite());
    assert!(Real::cbrt(f64::INFINITY) > 0.0);
    assert!(Real::cbrt(f64::NEG_INFINITY).is_infinite());
    assert!(Real::cbrt(f64::NEG_INFINITY) < 0.0);
    assert!(Real::cbrt(f64::NAN).is_nan());

    assert!(Real::cbrt(f32::INFINITY).is_infinite());
    assert!(Real::cbrt(f32::NAN).is_nan());
}

/// Unlike squaring, cubing overflows well before the type's maximum, so the extremes are where
/// a naive `powf`-based route loses the answer entirely.
#[test]
fn cbrt_at_the_representable_extremes_f64() {
    let big = Real::cbrt(f64::MAX);
    assert!(big.is_finite(), "cbrt(f64::MAX) = {big}");
    // cbrt(MAX)³ is MAX, which cannot be recomputed directly: the partial product
    // overflows. Dividing by MAX first keeps every intermediate in range, so the
    // invariant becomes cbrt(MAX)³ / MAX = 1.
    let recovered = (big / f64::MAX) * big * big;
    assert!(
        (recovered - 1.0).abs() <= 1e-9,
        "cbrt(MAX)³/MAX = {recovered}"
    );

    let small = Real::cbrt(f64::MIN_POSITIVE);
    assert!(small > 0.0, "cbrt(f64::MIN_POSITIVE) = {small}");
    assert!((small * small * small - f64::MIN_POSITIVE).abs() <= 1e-9 * f64::MIN_POSITIVE);

    assert_eq!(Real::cbrt(f64::MIN), -Real::cbrt(f64::MAX));
}

#[test]
fn cbrt_at_the_representable_extremes_f32() {
    let big = Real::cbrt(f32::MAX);
    assert!(big.is_finite(), "cbrt(f32::MAX) = {big}");
    let recovered = (big / f32::MAX) * big * big;
    assert!(
        (recovered - 1.0).abs() <= 1e-4,
        "cbrt(MAX)³/MAX = {recovered}"
    );

    let small = Real::cbrt(f32::MIN_POSITIVE);
    assert!(small > 0.0, "cbrt(f32::MIN_POSITIVE) = {small}");
}

/// `cbrt` is odd: cbrt(-x) = -cbrt(x). A symmetry the sign-branch workaround had to hand-code.
#[test]
fn cbrt_is_an_odd_function() {
    for x in [1e-8_f64, 0.25, 1.0, 3.0, 17.0, 1e8] {
        assert_eq!(Real::cbrt(-x), -Real::cbrt(x), "oddness at {x}");
    }
}

/// `cbrt` is strictly increasing across zero, which the `powf` route cannot express at all
/// because it is undefined on the negative half.
#[test]
fn cbrt_is_monotonic_across_zero() {
    let xs = [
        -1000.0_f64,
        -8.0,
        -1.0,
        -0.001,
        0.0,
        0.001,
        1.0,
        8.0,
        1000.0,
    ];
    for w in xs.windows(2) {
        assert!(
            Real::cbrt(w[0]) < Real::cbrt(w[1]),
            "cbrt({}) < cbrt({})",
            w[0],
            w[1]
        );
    }
}

/// The method is reachable through the bound, not only on the concrete types.
#[test]
fn cbrt_is_reachable_through_a_generic_real_bound() {
    fn analytic_cbrt<T: Real>(x: T) -> T {
        x.cbrt()
    }
    assert_eq!(analytic_cbrt(-27.0_f64), -3.0);
    assert_eq!(analytic_cbrt(-27.0_f32), -3.0);
    assert!(
        (analytic_cbrt(Float106::from(8.0)) - Float106::from(2.0)).abs() < Float106::from(1e-30)
    );
}

// --- RealField: ToPrimitive ---------------------------------------------------------------

/// The point of the supertrait: this function names `RealField` only, and still converts back
/// to a primitive. Before the change it needed `T: RealField + ToPrimitive`.
fn to_f64_through_realfield<T: RealField>(x: T) -> Option<f64> {
    x.to_f64()
}

fn to_i64_through_realfield<T: RealField>(x: T) -> Option<i64> {
    x.to_i64()
}

#[test]
fn realfield_converts_to_f64_without_restating_the_bound() {
    assert_eq!(to_f64_through_realfield(2.5_f64), Some(2.5));
    assert_eq!(to_f64_through_realfield(2.5_f32), Some(2.5));
    assert_eq!(to_f64_through_realfield(Float106::from(2.5)), Some(2.5));
}

#[test]
fn realfield_converts_to_an_integer_without_restating_the_bound() {
    // Truncation toward zero is `ToPrimitive`'s documented behaviour.
    assert_eq!(to_i64_through_realfield(2.9_f64), Some(2));
    assert_eq!(to_i64_through_realfield(-2.9_f64), Some(-2));
    assert_eq!(to_i64_through_realfield(Float106::from(-7.0)), Some(-7));
}

#[test]
fn realfield_integer_conversion_reports_failure_rather_than_substituting() {
    // The floor scan being retired used `unwrap_or_else(R::zero)` here. The conversion has a
    // `None` to report, and callers must see it.
    assert_eq!(to_i64_through_realfield(f64::NAN), None);
    assert_eq!(to_i64_through_realfield(f64::INFINITY), None);
    assert_eq!(to_i64_through_realfield(f64::MAX), None);
}

#[test]
fn every_concrete_scalar_satisfies_the_new_supertrait() {
    fn assert_to_primitive<T: RealField>() {}
    assert_to_primitive::<f32>();
    assert_to_primitive::<f64>();
    assert_to_primitive::<Float106>();
}
