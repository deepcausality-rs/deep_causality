/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Normed::modulus` at the non-finite components, where the scaled form needs explicit guards.
//!
//! The scaled form is `max · sqrt(1 + (min/max)²)`. Two inputs break it if the non-finite cases are
//! left to the arithmetic:
//!
//! * A `NaN` beside a zero. `NaN > 0` is false, so the ordering puts the zero in `max` and the
//!   zero-maximum guard returns zero — swallowing a `NaN` into a finite answer. That is worse than
//!   an unhelpful number: a residual whose entries are compared against a tolerance reports no
//!   defect at all.
//! * Two infinities. The ratio is `∞/∞ = NaN`, so a modulus that is genuinely infinite comes back
//!   undefined.
//!
//! `modulus_squared` gets both right, by summing squares — so the two disagree, and the pair is the
//! oracle: whatever `re² + im²` says about *finiteness*, `modulus` must agree with.
//!
//! The finite behaviour is pinned alongside, because a guard added in the wrong place would change
//! it: `|3 + 4i| = 5` from the Pythagorean triple, and the same triple scaled to where the squares
//! overflow, which is what the scaled form exists for.

use deep_causality_algebra::Normed;
use deep_causality_num_complex::Complex;

#[test]
fn test_a_nan_component_beside_a_zero_is_not_swallowed() {
    // `modulus_squared` is NaN here; the modulus must not disagree by returning a finite zero.
    let z = Complex::new(f64::NAN, 0.0);
    assert!(z.modulus_squared().is_nan(), "premise: the square is NaN");
    assert!(z.modulus().is_nan(), "got {}", z.modulus());
}

#[test]
fn test_a_nan_component_beside_a_zero_in_either_position() {
    assert!(Complex::new(0.0f64, f64::NAN).modulus().is_nan());
    assert!(Complex::new(f64::NAN, 0.0f64).modulus().is_nan());
}

#[test]
fn test_a_nan_component_beside_a_finite_one_is_nan() {
    assert!(Complex::new(f64::NAN, 1.0f64).modulus().is_nan());
    assert!(Complex::new(1.0f64, f64::NAN).modulus().is_nan());
}

#[test]
fn test_two_infinite_components_give_infinity_rather_than_nan() {
    // The ratio is inf/inf. The modulus of a point infinitely far out is infinite, not undefined.
    let z = Complex::new(f64::INFINITY, f64::INFINITY);
    assert!(
        z.modulus_squared().is_infinite(),
        "premise: the square is infinite"
    );
    let m = z.modulus();
    assert!(m.is_infinite() && m > 0.0, "got {m}");
}

#[test]
fn test_mixed_signed_infinities_give_positive_infinity() {
    let m = Complex::new(f64::NEG_INFINITY, f64::INFINITY).modulus();
    assert!(m.is_infinite() && m > 0.0, "got {m}");
}

#[test]
fn test_one_infinite_component_gives_infinity() {
    let m = Complex::new(f64::INFINITY, 1.0f64).modulus();
    assert!(m.is_infinite() && m > 0.0, "got {m}");
}

#[test]
fn test_a_nan_beside_an_infinity_is_nan() {
    // A NaN outranks an infinity, matching what summing the squares gives.
    assert!(Complex::new(f64::INFINITY, f64::NAN).modulus().is_nan());
}

#[test]
fn test_the_finite_modulus_is_unchanged() {
    // |3 + 4i| = 5, by hand.
    assert_eq!(Complex::new(3.0f64, 4.0).modulus(), 5.0);
    assert_eq!(Complex::new(-3.0f64, -4.0).modulus(), 5.0);
    assert_eq!(Complex::new(0.0f64, 0.0).modulus(), 0.0);
    assert_eq!(Complex::new(0.0f64, -7.5).modulus(), 7.5);
}

#[test]
fn test_the_scaled_form_still_beats_the_overflow() {
    // |3·2⁶⁰⁰ + 4·2⁶⁰⁰ i| = 5·2⁶⁰⁰, exact, while the squares reach infinity.
    let p = 2f64.powi(600);
    let z = Complex::new(3.0 * p, 4.0 * p);
    assert!(
        z.modulus_squared().is_infinite(),
        "premise: the square overflows"
    );
    assert_eq!(z.modulus(), 5.0 * p);
}

#[test]
fn test_the_scaled_form_still_beats_the_underflow() {
    // The same triple where the squares fall below the smallest subnormal.
    let p = 2f64.powi(-600);
    let z = Complex::new(3.0 * p, 4.0 * p);
    assert_eq!(z.modulus_squared(), 0.0, "premise: the square underflows");
    assert_eq!(z.modulus(), 5.0 * p);
}
