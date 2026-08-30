/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Witness for `lean/DeepCausalityFormal/Algebra/EuclideanDomain.lean`.
//!
//! Lean proves these laws over Mathlib's `ℤ` for all inputs; each test pins the crate's
//! `EuclideanDomain` implementation to the same statement at representative inputs. The
//! `THEOREM_MAP` ids match `lean/THEOREM_MAP.md`.
//!
//! The trait methods are called in fully-qualified form throughout: the inherent
//! `i64::div_euclid`/`rem_euclid` take their operand by value and win method resolution, so
//! `a.rem_euclid(&b)` would not exercise the trait.

use deep_causality_algebra::{CommutativeRing, EuclideanDomain, Field};
use deep_causality_num::One;

/// THEOREM_MAP: algebra.euclidean.remainder_nonneg
#[test]
fn test_remainder_nonneg() {
    // b ≠ 0 → 0 ≤ a % b. Unlike `%`, `rem_euclid` is non-negative whatever the signs.
    for a in -30_i64..=30 {
        for b in -7_i64..=7 {
            if b == 0 {
                continue;
            }
            assert!(
                EuclideanDomain::rem_euclid(&a, &b) >= 0,
                "negative remainder for a={a}, b={b}"
            );
        }
    }
}

/// THEOREM_MAP: algebra.euclidean.remainder_lt_divisor
#[test]
fn test_remainder_lt_divisor() {
    // b ≠ 0 → a % b < |b|. The termination argument: φ strictly decreases, and φ is ℕ-valued.
    //
    // Negative divisors are exercised alongside positive ones. The first step of the Euclidean
    // algorithm takes the caller's `other` as its divisor, and that argument may be negative;
    // only from the second step on is the divisor a previous remainder and therefore
    // non-negative. A bound stated for `0 < b` would leave that first step uncovered.
    for a in -30_i64..=30 {
        for b in -9_i64..=9 {
            if b == 0 {
                continue;
            }
            let r = EuclideanDomain::rem_euclid(&a, &b);
            assert!(
                r < b.abs(),
                "remainder not below |divisor| for a={a}, b={b}, r={r}"
            );
        }
    }
}

/// THEOREM_MAP: algebra.euclidean.remainder_lt_divisor
#[test]
fn test_euclidean_algorithm_terminates() {
    // The decreasing sequence the termination argument describes, run concretely: each remainder
    // is strictly smaller than |divisor|, so the recursion bottoms out. Run twice — once from a
    // positive divisor, once from a negative one, which is the case the `b ≠ 0` form of the
    // bound is needed for.
    fn run(mut a: i64, mut b: i64) -> i64 {
        let mut steps = 0;
        while b != 0 {
            let r = EuclideanDomain::rem_euclid(&a, &b);
            assert!(r < b.abs(), "phi did not decrease: a={a}, b={b}, r={r}");
            a = b;
            b = r;
            steps += 1;
            assert!(steps < 64, "Euclidean algorithm failed to terminate");
        }
        a
    }

    assert_eq!(run(1071, 462), 21);
    assert_eq!(run(1071, -462), 21);
}

/// THEOREM_MAP: algebra.euclidean.gcd_dvd_left
#[test]
fn test_gcd_divides_left() {
    // gcd(a, b) ∣ a.
    for a in -24_i64..=24 {
        for b in -12_i64..=12 {
            let g = a.gcd(&b);
            if g != 0 {
                assert_eq!(a % g, 0, "gcd {g} does not divide a={a} (b={b})");
            }
        }
    }
}

/// THEOREM_MAP: algebra.euclidean.gcd_dvd_right
#[test]
fn test_gcd_divides_right() {
    // gcd(a, b) ∣ b.
    for a in -24_i64..=24 {
        for b in -12_i64..=12 {
            let g = a.gcd(&b);
            if g != 0 {
                assert_eq!(b % g, 0, "gcd {g} does not divide b={b} (a={a})");
            }
        }
    }
}

/// THEOREM_MAP: algebra.euclidean.gcd_nonneg
#[test]
fn test_gcd_nonneg() {
    // 0 ≤ gcd(a, b), whatever the signs — the algorithm iterates the non-negative `rem_euclid`.
    assert_eq!((-48_i64).gcd(&18), 6);
    assert_eq!(48_i64.gcd(&-18), 6);
    assert_eq!((-48_i64).gcd(&-18), 6);
    for a in -24_i64..=24 {
        for b in -12_i64..=12 {
            assert!(a.gcd(&b) >= 0, "negative gcd for a={a}, b={b}");
        }
    }

    // Where the Lean statement and the Rust type part company. Lean proves this over ℤ, which is
    // unbounded, so `0 ≤ gcd a b` holds for every pair. `i64` is not ℤ: at `i64::MIN` the
    // canonical associate `|MIN|` is unrepresentable, so the total `gcd` cannot return a
    // non-negative value there — it panics in debug and wraps to a negative one in release.
    // `checked_gcd` is the total form, and reports that pair rather than violating the bound.
    assert_eq!(EuclideanDomain::checked_gcd(&i64::MIN, &0_i64), None);
    // Everywhere the result is representable, the checked form agrees and the bound holds.
    assert_eq!(EuclideanDomain::checked_gcd(&i64::MIN, &6_i64), Some(2));
}

/// THEOREM_MAP: algebra.euclidean.gcd_zero_right
#[test]
fn test_gcd_zero_right() {
    // gcd(a, 0) = |a| — the base case where the recursion stops.
    assert_eq!(7_i64.gcd(&0), 7);
    assert_eq!((-7_i64).gcd(&0), 7);
    assert_eq!(0_i64.gcd(&0), 0);
}

/// THEOREM_MAP: algebra.euclidean.integers_not_field
#[test]
fn test_integers_are_not_a_field() {
    // ¬∃ x : ℤ, 2·x = 1. Integer division truncates rather than inverting, so no integer is a
    // multiplicative inverse of 2.
    assert!(
        !(-1000_i64..=1000).any(|x| 2 * x == 1),
        "found an integer inverse of 2"
    );
    // The truncation itself: 1/5 is 0, so 5 · (1/5) is 0 rather than 1. Read through variables
    // so the expression is evaluated rather than constant-folded.
    for d in 2_i64..=9 {
        let one = i64::one();
        let reciprocal = one / d;
        assert_eq!(reciprocal, 0, "1/{d} did not truncate to zero");
        assert_ne!(d * reciprocal, one, "{d} · (1/{d}) must not be 1 over ℤ");
    }

    // The tower records exactly this. ℤ reaches CommutativeRing:
    fn assert_commutative_ring<T: CommutativeRing>() {}
    assert_commutative_ring::<i64>();
    // ...and stops there. `fn assert_field<T: Field>() {}; assert_field::<i64>()` does not
    // compile, because the `Invertible` marker is withheld from the integers.
    fn assert_field<T: Field>() {}
    assert_field::<f64>();
}
