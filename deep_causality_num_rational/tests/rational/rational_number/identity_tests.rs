/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The two identities, `0/1` and `1/1`, and the predicates that recognise them.
//!
//! `is_zero` and `is_one` are direct checks on the stored components rather than comparisons
//! against a constructed value, which is only sound because the form is canonical: no other pair
//! represents zero or one. Every case below therefore goes through a value that had to be
//! *reduced* to the identity, since that is the path where a non-canonical representation would
//! show up.

use deep_causality_num::{One, Zero};
use deep_causality_num_rational::Rational;

#[test]
fn is_one_recognises_the_multiplicative_identity() {
    assert!(Rational::<i64>::one().is_one());
    assert!(Rational::new(1_i64, 1).is_one());

    // The case that actually exercises canonical form: `7/7` is stored as `1/1`, so a direct
    // component check recognises it without any cross-multiplication.
    assert!(Rational::new(7_i64, 7).is_one());
    assert!(Rational::new(-7_i64, -7).is_one());
    assert!(Rational::new(i64::MAX, i64::MAX).is_one());

    // And a value that arrives at one through the arithmetic rather than the constructor.
    assert!((Rational::new(2_i64, 3) * Rational::new(3_i64, 2)).is_one());
    assert!((Rational::new(3_i64, 4) / Rational::new(3_i64, 4)).is_one());
}

#[test]
fn is_one_rejects_everything_else() {
    assert!(!Rational::new(1_i64, 2).is_one(), "a proper fraction");
    assert!(
        !Rational::new(2_i64, 1).is_one(),
        "an integer other than one"
    );
    assert!(!Rational::<i64>::zero().is_one(), "zero");
    assert!(
        !Rational::new(-1_i64, 1).is_one(),
        "the additive inverse of one"
    );
    assert!(!Rational::new(1_i64, i64::MAX).is_one(), "a tiny value");
    assert!(
        !Rational::new(i64::MAX, 1).is_one(),
        "a large integer, where a numerator-only check would still say no"
    );
}

#[test]
fn is_zero_recognises_the_additive_identity() {
    assert!(Rational::<i64>::zero().is_zero());
    assert!(Rational::new(0_i64, 7).is_zero(), "0/7 is stored as 0/1");
    assert!(Rational::new(0_i64, -7).is_zero());
    assert!(
        !Rational::new(1_i64, i64::MAX).is_zero(),
        "a tiny value is not zero"
    );
    assert!(!Rational::<i64>::one().is_zero());
}

#[test]
fn default_is_the_additive_identity() {
    let d = Rational::<i64>::default();
    assert_eq!(d, Rational::zero());
    assert!(d.is_zero());
    assert!(!d.is_one());
    assert_eq!(*d.numer(), 0);
    assert_eq!(*d.denom(), 1, "zero is canonically 0/1, not 0/anything");

    // The same at every width the type is built over.
    assert_eq!(Rational::<i8>::default(), Rational::zero());
    assert_eq!(Rational::<i128>::default(), Rational::zero());
    assert_eq!(Rational::<isize>::default(), Rational::zero());
}

#[test]
fn the_predicates_round_trip_with_their_constructors() {
    let zero = Rational::<i64>::zero();
    let one = Rational::<i64>::one();

    assert!(zero.is_zero());
    assert!(one.is_one());
    assert!(!zero.is_one());
    assert!(!one.is_zero());
    assert_ne!(zero, one);

    // The identities behave as identities, which is the reason the predicates exist.
    let a = Rational::new(5_i64, 8);
    assert_eq!(a + zero, a);
    assert_eq!(a * one, a);
    assert!((a - a).is_zero());
    assert!((a / a).is_one());
}
