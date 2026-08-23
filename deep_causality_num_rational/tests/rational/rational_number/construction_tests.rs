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
