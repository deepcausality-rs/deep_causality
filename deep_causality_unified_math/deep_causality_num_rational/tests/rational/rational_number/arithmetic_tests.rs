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
    // `big` exceeds sqrt(i64::MAX) — 3_037_000_500² is 9_223_372_037_000_250_000, just past
    // i64::MAX at 9_223_372_036_854_775_807 — so a naive `b * d` on a common denominator would
    // overflow immediately. Cancelling the shared factor first keeps the denominator at `big`
    // throughout. The previous constant here, 3_037_000_499, was one step too small: its square
    // is 9_223_372_030_926_249_001, which still fits, so the test passed without exercising the
    // property it names.
    let big = 3_037_000_500_i64;
    assert!(
        i128::from(big) * i128::from(big) > i128::from(i64::MAX),
        "the constant must exceed sqrt(i64::MAX), or the naive product would not overflow and \
         this test would pass without exercising the property it names"
    );
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

#[test]
fn negation_is_total_because_the_numerator_is_never_the_signed_minimum() {
    // The regression this pins: `T::MIN` used to reach the numerator, and `-Rational::new(
    // i64::MIN, 1)` then overflowed — a panic in debug, `i64::MIN` again in release, which would
    // have made `a + (-a) == 0` false. Construction now refuses that one value per width, so
    // negation has nothing left to fail on.
    assert_eq!(
        Rational::try_new(i64::MIN, 1),
        None,
        "T::MIN must not reach a numerator"
    );
    assert_eq!(Rational::try_from_integer(i8::MIN), None);

    // Every value that *is* constructible negates, negation is an involution, and the additive
    // inverse law holds at the edge of the range as well as in the middle.
    for n in [i64::MIN + 1, -7, -1, 0, 1, 7, i64::MAX] {
        let r = Rational::new(n, 1);
        assert_eq!(*(-r).numer(), -n, "negation of {n}");
        assert_eq!(-(-r), r, "negation is an involution at {n}");
        assert_eq!(r + (-r), Rational::zero(), "no additive inverse for {n}");
        assert_eq!(r - r, Rational::zero(), "subtraction fails at {n}");
    }

    // The extreme numerator a canonical form can hold, at the narrowest width.
    let r = Rational::new(i8::MIN + 1, 1);
    assert_eq!(*(-r).numer(), i8::MAX);
    // And with a denominator, where reduction has already been through the same edge.
    let r = Rational::new(i64::MIN + 1, 2);
    assert_eq!(*(-r).numer(), i64::MAX);
    assert_eq!(*(-r).denom(), 2);
}

#[test]
fn addition_carries_the_integer_part_instead_of_overflowing_the_numerator() {
    // The regression this pins: `MAX/2 + MAX/2` is exactly `MAX`, which fits — but the direct
    // numerator sum `MAX + MAX` does not, so the old implementation panicked in debug and wrapped
    // to `-1` in release. Splitting each operand into an integer part and a proper fraction means
    // the large part is carried as an integer and never multiplied back out.
    let half_max = Rational::new(i64::MAX, 2);
    assert_eq!(half_max + half_max, Rational::from_integer(i64::MAX));

    // The same at the narrowest width: 127/2 + 127/2 = 127.
    let half_max_i8 = Rational::new(i8::MAX, 2);
    assert_eq!(half_max_i8 + half_max_i8, Rational::from_integer(i8::MAX));

    // A fractional part that survives the carry: MAX/4 + MAX/4 = MAX/2, numerator MAX.
    let quarter_max = Rational::new(i64::MAX, 4);
    assert_eq!(quarter_max + quarter_max, half_max);

    // Negative operands take the mirrored path through the same carry.
    assert_eq!(
        -half_max + -half_max,
        -Rational::from_integer(i64::MAX),
        "the negative half of the range carries too"
    );

    // Subtraction inherits it, being addition of the negation.
    assert_eq!(half_max - -half_max, Rational::from_integer(i64::MAX));

    // What is *not* fixed, stated as a test rather than left to the reader: the limit is on the
    // intermediates, not on the answer. Four quarters summed as a tree reach MAX, because no
    // intermediate exceeds it; summed left to right they pass through 3·MAX/4, whose numerator
    // 3·MAX does not fit in an i64 at all. No arrangement of the arithmetic can represent that
    // intermediate — a wider `T` is the only remedy.
    let left = quarter_max + quarter_max;
    let right = quarter_max + quarter_max;
    assert_eq!(left + right, Rational::from_integer(i64::MAX));
}

#[test]
fn addition_matches_exact_arithmetic_computed_in_a_wider_width() {
    // Cross-check against `i128`, which has room for the naive formula an `i32` does not. This is
    // what catches a *wrong value* in release, where the old code wrapped rather than panicking.
    fn gcd(mut a: i128, mut b: i128) -> i128 {
        a = a.abs();
        b = b.abs();
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        a
    }

    let nums = [0_i32, 1, -1, 7, -7, i32::MAX, i32::MAX - 1, i32::MIN + 1];
    let dens = [1_i32, 2, 3, 4, 6, 8, 12, 997];
    let (lo, hi) = (i128::from(i32::MIN) + 1, i128::from(i32::MAX));

    for &an in &nums {
        for &ad in &dens {
            for &bn in &nums {
                for &bd in &dens {
                    let a = Rational::new(an, ad);
                    let b = Rational::new(bn, bd);
                    let (a_n, a_d) = (i128::from(*a.numer()), i128::from(*a.denom()));
                    let (b_n, b_d) = (i128::from(*b.numer()), i128::from(*b.denom()));

                    let mut num = a_n * b_d + b_n * a_d;
                    let mut den = a_d * b_d;
                    let g = gcd(num, den);
                    num /= g;
                    den /= g;

                    // Addition promises a correct answer when the answer is representable and the
                    // pieces it is assembled from are — the integer parts it sums, and the
                    // fractional numerator bounded by twice the least common denominator. Both
                    // bounds are documented on `Add`; the cases outside them are what a wider `T`
                    // is for.
                    let int_parts = a_n.div_euclid(a_d) + b_n.div_euclid(b_d);
                    if num < lo || num > hi || den > hi || int_parts < lo || int_parts > hi {
                        continue;
                    }

                    let sum = a + b;
                    assert_eq!(
                        (i128::from(*sum.numer()), i128::from(*sum.denom())),
                        (num, den),
                        "{an}/{ad} + {bn}/{bd}"
                    );
                    assert_eq!(sum, b + a, "addition disagreed with itself commuted");
                }
            }
        }
    }
}
