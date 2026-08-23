/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Construction and the canonical-form invariants: positive denominator, coprime components,
//! zero as `0/1`.

use deep_causality_num::{One, Zero};
use deep_causality_num_rational::Rational;

#[test]
fn fractions_are_reduced_on_construction() {
    assert_eq!(Rational::new(6_i64, 8), Rational::new(3, 4));
    assert_eq!(Rational::new(100_i64, 10), Rational::from_integer(10));
    assert_eq!(Rational::new(-6_i64, 8), Rational::new(-3, 4));
}

#[test]
fn the_sign_never_lives_in_the_denominator() {
    let r = Rational::new(1_i64, -2);
    assert_eq!(*r.numer(), -1);
    assert_eq!(*r.denom(), 2);

    // Two negatives cancel rather than accumulate.
    let s = Rational::new(-3_i64, -6);
    assert_eq!(*s.numer(), 1);
    assert_eq!(*s.denom(), 2);
}

#[test]
fn zero_canonicalises_to_zero_over_one() {
    let z = Rational::new(0_i64, 7);
    assert_eq!(*z.numer(), 0);
    assert_eq!(*z.denom(), 1);
    assert_eq!(z, Rational::zero());
    assert!(z.is_zero());
}

#[test]
fn a_zero_denominator_has_no_value() {
    assert_eq!(Rational::try_new(1_i64, 0), None);
    assert_eq!(Rational::try_new(0_i64, 0), None);
    // A non-zero denominator always yields a value.
    assert!(Rational::try_new(1_i64, 2).is_some());
}

#[test]
#[should_panic(expected = "zero denominator")]
fn new_panics_on_a_zero_denominator() {
    let _ = Rational::new(1_i64, 0);
}

#[test]
fn from_integer_embeds_z_into_q() {
    let n = Rational::from_integer(42_i64);
    assert!(n.is_integer());
    assert_eq!(*n.numer(), 42);
    assert_eq!(*n.denom(), 1);
    // The embedding is injective.
    assert_ne!(Rational::from_integer(1_i64), Rational::from_integer(2_i64));
}

#[test]
fn is_integer_detects_a_unit_denominator() {
    assert!(Rational::new(4_i64, 2).is_integer());
    assert!(!Rational::new(3_i64, 2).is_integer());
    assert!(Rational::<i64>::one().is_integer());
}

#[test]
fn every_signed_width_can_carry_a_rational() {
    assert_eq!(*Rational::new(1_i8, 2).denom(), 2);
    assert_eq!(*Rational::new(1_i16, 2).denom(), 2);
    assert_eq!(*Rational::new(1_i32, 2).denom(), 2);
    assert_eq!(*Rational::new(1_i64, 2).denom(), 2);
    assert_eq!(*Rational::new(1_i128, 2).denom(), 2);
    assert_eq!(*Rational::new(1_isize, 2).denom(), 2);
}

#[test]
fn the_signed_minimum_reduces_before_the_sign_moves() {
    // The regression this pins: `reduce` used to negate both components whenever the denominator
    // was negative, *before* dividing by the gcd. For `i64::MIN / -2` that formed `-i64::MIN`,
    // which panicked in debug and, in release, returned `-2⁶²` — the negation of the right
    // answer. Dividing by the gcd first shrinks the magnitude to one the negation can hold.
    let r = Rational::try_new(i64::MIN, -2).expect("2^62 is representable");
    assert_eq!(*r.numer(), 4_611_686_018_427_387_904);
    assert_eq!(*r.denom(), 1);

    // The same shape one width down, where the arithmetic is checkable by eye: -128 / -2 = 64.
    let r = Rational::try_new(i8::MIN, -2).expect("64 fits in an i8");
    assert_eq!(*r.numer(), 64);
    assert_eq!(*r.denom(), 1);

    // With the minimum in the denominator instead: 4 / -2⁶³ = -1 / 2⁶¹.
    let r = Rational::try_new(4_i64, i64::MIN).expect("2^61 is representable");
    assert_eq!(*r.numer(), -1);
    assert_eq!(*r.denom(), 2_305_843_009_213_693_952);

    // MIN/MIN is one, and reaching that answer must not form gcd(MIN, MIN) = 2⁶³.
    assert_eq!(
        Rational::try_new(i64::MIN, i64::MIN),
        Some(Rational::from_integer(1_i64))
    );
    // Zero over the minimum is zero, and must not form gcd(0, MIN) = 2⁶³ either.
    assert_eq!(Rational::try_new(0_i64, i64::MIN), Some(Rational::zero()));
}

#[test]
fn values_with_no_canonical_form_are_rejected_rather_than_wrong() {
    // +2⁶³ is not representable at all.
    assert_eq!(Rational::try_new(i64::MIN, -1), None);
    // Nor is a denominator of 2⁶³.
    assert_eq!(Rational::try_new(3_i64, i64::MIN), None);
    // `i64::MIN` *is* representable as a value, and is refused anyway: invariant 4 keeps it out of
    // the numerator so that negation stays total.
    assert_eq!(Rational::try_new(i64::MIN, 1), None);
    assert_eq!(Rational::try_from_integer(i64::MIN), None);
    assert_eq!(Rational::try_from_integer(i8::MIN), None);

    // Everything one step inside the range is fine, so the loss is exactly one value per width.
    assert!(Rational::try_new(i64::MIN + 1, 1).is_some());
    assert!(Rational::try_from_integer(i64::MIN + 1).is_some());
    assert!(Rational::try_from_integer(i64::MAX).is_some());
}

#[test]
#[should_panic(expected = "no canonical form")]
fn new_panics_on_a_value_with_no_canonical_form() {
    let _ = Rational::new(i64::MIN, 1);
}

#[test]
#[should_panic(expected = "T::MIN")]
fn from_integer_panics_on_the_signed_minimum() {
    let _ = Rational::from_integer(i64::MIN);
}

#[test]
fn construction_is_total_across_the_edges_of_a_narrow_width() {
    // Every pair an `i8` admits either has a correct canonical form or none — never a wrong one.
    // Checked against exact `i32` arithmetic, so a release build that wraps is caught too.
    for n in i8::MIN..=i8::MAX {
        for d in i8::MIN..=i8::MAX {
            let got = Rational::try_new(n, d);
            if d == 0 {
                assert_eq!(got, None, "{n}/{d}");
                continue;
            }
            let (mut en, mut ed) = (i32::from(n), i32::from(d));
            let g = {
                let (mut a, mut b) = (en.abs(), ed.abs());
                while b != 0 {
                    let r = a % b;
                    a = b;
                    b = r;
                }
                a.max(1)
            };
            en /= g;
            ed /= g;
            if ed < 0 {
                en = -en;
                ed = -ed;
            }
            // Invariant 4 excludes a numerator of i8::MIN, hence the strict lower bound.
            let representable =
                en > i32::from(i8::MIN) && en <= i32::from(i8::MAX) && ed <= i32::from(i8::MAX);
            match got {
                Some(r) => {
                    assert!(representable, "{n}/{d} should have been rejected");
                    assert_eq!(i32::from(*r.numer()), en, "{n}/{d}");
                    assert_eq!(i32::from(*r.denom()), ed, "{n}/{d}");
                }
                None => assert!(
                    !representable,
                    "{n}/{d} has a canonical form but was rejected"
                ),
            }
        }
    }
}

#[test]
#[should_panic(expected = "no canonical form")]
fn arithmetic_panics_when_the_result_has_no_canonical_form() {
    // The operators return `Self` and so have nowhere to put a `None`. Invariant 4 forbids a
    // `T::MIN` numerator, and an exact result can land on it: (MIN+1) + (-1) is MIN/1, whose
    // negation is unrepresentable. That is the one arithmetic input the type cannot answer, and
    // it panics rather than wrapping to a wrong value.
    let a = Rational::new(i64::MIN + 1, 1);
    let b = Rational::from_integer(-1_i64);
    let _ = a + b;
}

#[test]
fn the_same_result_is_reportable_through_the_total_constructor() {
    // The value that arithmetic cannot represent is exactly the one `try_new` rejects, so a
    // caller who needs totality has a path that does not panic.
    assert_eq!(Rational::try_new(i64::MIN, 1), None);
    assert_eq!(Rational::try_from_integer(i64::MIN), None);
    // and one step away from it is fine
    assert!(Rational::try_new(i64::MIN + 1, 1).is_some());
}
