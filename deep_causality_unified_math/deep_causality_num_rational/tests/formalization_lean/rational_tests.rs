/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Rational/Rational.lean`.

use deep_causality_num::{One, Zero};
use deep_causality_num_rational::Rational;

type Q = Rational<i64>;

/// THEOREM_MAP: rational.field.mul_inv
#[test]
fn test_mul_inv() {
    // q ≠ 0 → q · q⁻¹ = 1. The defining field axiom, and the one ℤ cannot satisfy.
    for n in -8_i64..=8 {
        for d in 1_i64..=8 {
            if n == 0 {
                continue;
            }
            let q = Rational::new(n, d);
            assert_eq!(q * q.recip(), Q::one(), "no inverse for {n}/{d}");
        }
    }
    // Zero is the single exception, and the API says so rather than panicking silently.
    assert_eq!(Q::zero().checked_recip(), None);
}

/// THEOREM_MAP: rational.field.mul_comm
#[test]
fn test_mul_comm() {
    let a = Rational::new(3_i64, 7);
    let b = Rational::new(-2_i64, 5);
    assert_eq!(a * b, b * a);
}

/// THEOREM_MAP: rational.field.mul_assoc
#[test]
fn test_mul_assoc() {
    let a = Rational::new(3_i64, 7);
    let b = Rational::new(-2_i64, 5);
    let c = Rational::new(11_i64, 3);
    assert_eq!((a * b) * c, a * (b * c));
}

/// THEOREM_MAP: rational.field.distrib
#[test]
fn test_distrib() {
    let a = Rational::new(3_i64, 7);
    let b = Rational::new(-2_i64, 5);
    let c = Rational::new(11_i64, 3);
    assert_eq!(a * (b + c), a * b + a * c);
}

/// THEOREM_MAP: rational.abelian_group.add_neg
#[test]
fn test_add_neg() {
    // a + (−a) = 0, with the denominator left positive by the `Neg` impl.
    for n in -8_i64..=8 {
        for d in 1_i64..=8 {
            let a = Rational::new(n, d);
            assert_eq!(a + (-a), Q::zero(), "no additive inverse for {n}/{d}");
            assert!(
                *(-a).denom() > 0,
                "negation moved the sign into the denominator"
            );
        }
    }
}

/// THEOREM_MAP: rational.canonical.den_pos
#[test]
fn test_denominator_is_positive() {
    // Invariant 1: a sign never survives in the denominator.
    for n in -8_i64..=8 {
        for d in -8_i64..=8 {
            if d == 0 {
                continue;
            }
            let q = Rational::new(n, d);
            assert!(*q.denom() > 0, "non-positive denominator from {n}/{d}");
        }
    }
}

/// THEOREM_MAP: rational.canonical.coprime
#[test]
fn test_numerator_and_denominator_are_coprime() {
    // Invariant 2: components are reduced by their gcd, so the representation is unique. That
    // uniqueness is what makes equality structural rather than a cross-multiplication.
    fn gcd(mut a: i64, mut b: i64) -> i64 {
        a = a.abs();
        b = b.abs();
        while b != 0 {
            let r = a % b;
            a = b;
            b = r;
        }
        a
    }
    for n in -12_i64..=12 {
        for d in 1_i64..=12 {
            let q = Rational::new(n, d);
            assert_eq!(
                gcd(*q.numer(), *q.denom()),
                1,
                "not in lowest terms: {n}/{d} stored as {}/{}",
                q.numer(),
                q.denom()
            );
        }
    }
    // Structural equality follows from uniqueness.
    assert_eq!(Rational::new(6_i64, 8), Rational::new(3_i64, 4));
}

/// THEOREM_MAP: rational.order.dense
#[test]
fn test_density() {
    // Between any two distinct rationals lies a third — ℚ has no successor function, which is
    // the order property that separates it from ℤ.
    let two = Rational::from_integer(2_i64);
    let pairs = [
        (Rational::new(1_i64, 3), Rational::new(1_i64, 2)),
        (Rational::new(-1_i64, 2), Rational::new(1_i64, 2)),
        (Rational::new(22_i64, 7), Rational::new(23_i64, 7)),
    ];
    for (a, b) in pairs {
        assert!(a < b);
        let mid = (a + b) / two;
        assert!(
            a < mid && mid < b,
            "midpoint not strictly between {a} and {b}"
        );
    }
}
