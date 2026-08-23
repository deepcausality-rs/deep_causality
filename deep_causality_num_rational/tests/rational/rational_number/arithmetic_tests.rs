/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Field arithmetic, and the exactness that is the whole point of the type.

use deep_causality_num::{One, Zero};
use deep_causality_num_rational::Rational;

#[test]
fn addition_is_exact_where_floating_point_is_not() {
    let third = Rational::new(1_i64, 3);
    assert_eq!(third + third + third, Rational::one());

    // The canonical binary-float failure, which ℚ simply does not have.
    assert_ne!(0.1_f64 + 0.2_f64, 0.3_f64);
    assert_eq!(
        Rational::new(1_i64, 10) + Rational::new(2_i64, 10),
        Rational::new(3_i64, 10)
    );
}

#[test]
fn addition_uses_the_least_common_denominator() {
    // 1/6 + 1/4 = 5/12, not 10/24: the shared factor 2 is cancelled before multiplying.
    let sum = Rational::new(1_i64, 6) + Rational::new(1_i64, 4);
    assert_eq!(*sum.numer(), 5);
    assert_eq!(*sum.denom(), 12);
}

#[test]
fn subtraction_is_addition_of_the_negation() {
    let a = Rational::new(3_i64, 4);
    let b = Rational::new(1_i64, 4);
    assert_eq!(a - b, Rational::new(1_i64, 2));
    assert_eq!(a - a, Rational::zero());
    assert_eq!(b - a, Rational::new(-1_i64, 2));
}

#[test]
fn multiplication_cross_cancels() {
    // 2/3 · 3/2 = 1 exactly, with no intermediate 6/6.
    assert_eq!(
        Rational::new(2_i64, 3) * Rational::new(3_i64, 2),
        Rational::one()
    );
    assert_eq!(
        Rational::new(3_i64, 4) * Rational::new(2_i64, 5),
        Rational::new(3_i64, 10)
    );
}

#[test]
fn multiplication_by_zero_annihilates() {
    let a = Rational::new(7_i64, 9);
    assert_eq!(a * Rational::zero(), Rational::zero());
    assert_eq!(Rational::zero() * a, Rational::<i64>::zero());
}

#[test]
fn division_is_multiplication_by_the_reciprocal() {
    let a = Rational::new(3_i64, 4);
    let b = Rational::new(2_i64, 5);
    assert_eq!(a / b, a * b.recip());
    assert_eq!(a / b, Rational::new(15_i64, 8));
    assert_eq!(a / a, Rational::one());
}

#[test]
#[should_panic(expected = "division by zero")]
fn division_by_zero_panics() {
    let _ = Rational::new(1_i64, 2) / Rational::zero();
}

#[test]
fn negation_preserves_the_invariants() {
    let a = Rational::new(3_i64, 4);
    let n = -a;
    assert_eq!(*n.numer(), -3);
    assert_eq!(*n.denom(), 4, "the denominator stays positive");
    assert_eq!(-n, a, "negation is an involution");
    assert_eq!(-Rational::<i64>::zero(), Rational::zero());
}

#[test]
fn every_non_zero_element_has_an_inverse() {
    // The field axiom, checked across a spread of values.
    for n in -10_i64..=10 {
        for d in 1_i64..=10 {
            if n == 0 {
                continue;
            }
            let r = Rational::new(n, d);
            assert_eq!(r * r.recip(), Rational::one(), "failed for {n}/{d}");
        }
    }
}

#[test]
fn zero_is_the_one_element_without_an_inverse() {
    assert_eq!(Rational::<i64>::zero().checked_recip(), None);
    assert!(Rational::new(1_i64, 2).checked_recip().is_some());
}

#[test]
#[should_panic(expected = "recip called on zero")]
fn recip_panics_on_zero() {
    let _ = Rational::<i64>::zero().recip();
}

#[test]
fn assign_operators_agree_with_their_binary_forms() {
    let a = Rational::new(3_i64, 4);
    let b = Rational::new(1_i64, 6);

    let mut x = a;
    x += b;
    assert_eq!(x, a + b);

    let mut x = a;
    x -= b;
    assert_eq!(x, a - b);

    let mut x = a;
    x *= b;
    assert_eq!(x, a * b);

    let mut x = a;
    x /= b;
    assert_eq!(x, a / b);
}

#[test]
fn sum_and_product_fold_over_the_identities() {
    let terms = [
        Rational::new(1_i64, 2),
        Rational::new(1_i64, 3),
        Rational::new(1_i64, 6),
    ];
    let total: Rational<i64> = terms.into_iter().sum();
    assert_eq!(total, Rational::one());

    let factors = [Rational::new(2_i64, 3), Rational::new(3_i64, 2)];
    let product: Rational<i64> = factors.into_iter().product();
    assert_eq!(product, Rational::one());

    // Empty folds give the identities.
    let empty: Rational<i64> = core::iter::empty::<Rational<i64>>().sum();
    assert_eq!(empty, Rational::zero());
    let empty: Rational<i64> = core::iter::empty::<Rational<i64>>().product();
    assert_eq!(empty, Rational::one());
}

#[test]
fn cross_cancellation_delays_overflow() {
    // `big` exceeds sqrt(i64::MAX), so a naive `b * d` on a common denominator would overflow
    // immediately. Cancelling the shared factor first keeps the denominator at `big` throughout.
    let big = 3_037_000_499_i64;
    let x = Rational::new(1_i64, big);
    let mut acc = Rational::zero();
    for _ in 0..64 {
        acc += x;
    }
    assert_eq!(acc, Rational::new(64_i64, big));
}

#[test]
fn arithmetic_is_associative_and_commutative() {
    let a = Rational::new(1_i64, 3);
    let b = Rational::new(2_i64, 5);
    let c = Rational::new(3_i64, 7);

    assert_eq!((a + b) + c, a + (b + c));
    assert_eq!(a + b, b + a);
    assert_eq!((a * b) * c, a * (b * c));
    assert_eq!(a * b, b * a);
    // Distributivity.
    assert_eq!(a * (b + c), a * b + a * c);
}
