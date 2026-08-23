/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `NaturalNumber`, the set-named entry point for ℕ.
//!
//! These exercise the trait through a generic bound rather than through the inherent unsigned
//! methods, so what is under test is the abstraction.

use deep_causality_num::NaturalNumber;

fn succ_of<T: NaturalNumber>(n: T) -> Option<T> {
    n.succ()
}
fn gcd_of<T: NaturalNumber>(a: T, b: T) -> T {
    a.gcd(b)
}
fn lcm_of<T: NaturalNumber>(a: T, b: T) -> Option<T> {
    a.lcm(b)
}

#[test]
fn every_unsigned_width_is_a_natural_number() {
    fn assert_natural<T: NaturalNumber>() {}
    assert_natural::<u8>();
    assert_natural::<u16>();
    assert_natural::<u32>();
    assert_natural::<u64>();
    assert_natural::<u128>();
    assert_natural::<usize>();
}

#[test]
fn successor_is_total_in_n_but_bounded_in_the_carrier() {
    assert_eq!(succ_of(3u64), Some(4));
    assert_eq!(succ_of(0u64), Some(1));
    // `None` here is a limit of the representation, not a fact about ℕ: every natural number
    // has a successor.
    assert_eq!(succ_of(u64::MAX), None);
    assert_eq!(succ_of(u8::MAX), None);
}

#[test]
fn zero_has_no_predecessor() {
    // Unlike `succ`, this `None` is genuine ℕ partiality at every width — the Peano base case.
    assert_eq!(3u64.pred(), Some(2));
    assert_eq!(1u64.pred(), Some(0));
    assert_eq!(0u64.pred(), None);
    assert_eq!(0u8.pred(), None);
    assert_eq!(0usize.pred(), None);
}

#[test]
fn checked_difference_reports_the_absent_difference() {
    assert_eq!(5u64.checked_difference(3), Some(2));
    assert_eq!(3u64.checked_difference(3), Some(0));
    // `3 - 5` has no value in ℕ, and the type says so rather than wrapping.
    assert_eq!(3u64.checked_difference(5), None);
    assert_eq!(0u64.checked_difference(1), None);
}

#[test]
fn monus_is_total() {
    // Truncated subtraction `a ∸ b`: the difference where it exists, zero elsewhere.
    assert_eq!(5u64.monus(3), 2);
    assert_eq!(3u64.monus(5), 0);
    assert_eq!(0u64.monus(7), 0);
    // Total for every pair, which is what makes it the standard operation on ℕ. Checked against
    // the defining property rather than against `saturating_sub`, which is the implementation:
    // for `b ≤ a` the result is the unique `c` with `b + c = a`, and zero otherwise.
    for a in 0u8..=20 {
        for b in 0u8..=20 {
            let m = a.monus(b);
            if b > a {
                assert_eq!(m, 0, "{a} ∸ {b} must truncate to zero");
            } else {
                assert_eq!(b + m, a, "{a} ∸ {b} must satisfy b + (a ∸ b) = a");
            }
        }
    }
}

#[test]
fn div_rem_is_partial_at_zero() {
    assert_eq!(17u64.div_rem(5), Some((3, 2)));
    assert_eq!(20u64.div_rem(4), Some((5, 0)));
    assert_eq!(0u64.div_rem(4), Some((0, 0)));
    assert_eq!(17u64.div_rem(0), None);
}

#[test]
fn div_rem_reconstructs_the_dividend() {
    for a in 0u32..=60 {
        for b in 1u32..=9 {
            let (q, r) = a.div_rem(b).expect("non-zero divisor");
            assert_eq!(a, b * q + r, "reconstruction failed for {a}, {b}");
            assert!(r < b, "remainder not below divisor for {a}, {b}");
        }
    }
}

#[test]
fn gcd_needs_no_normalization() {
    // With no sign there is no choice of associate, so the result is already canonical —
    // unlike the signed case, which has to normalize.
    assert_eq!(gcd_of(48u64, 18u64), 6);
    assert_eq!(gcd_of(1071u64, 462u64), 21);
    assert_eq!(gcd_of(17u64, 5u64), 1);
    assert_eq!(gcd_of(13u64, 13u64), 13);
}

#[test]
fn gcd_base_cases() {
    assert_eq!(gcd_of(7u64, 0u64), 7);
    assert_eq!(gcd_of(0u64, 7u64), 7);
    assert_eq!(gcd_of(0u64, 0u64), 0);
}

#[test]
fn gcd_divides_both_arguments() {
    for a in 1u32..=40 {
        for b in 1u32..=40 {
            let g = gcd_of(a, b);
            assert_eq!(a % g, 0, "gcd {g} does not divide {a}");
            assert_eq!(b % g, 0, "gcd {g} does not divide {b}");
        }
    }
}

#[test]
fn lcm_divides_before_multiplying() {
    assert_eq!(lcm_of(4u64, 6u64), Some(12));
    assert_eq!(lcm_of(21u64, 6u64), Some(42));
    // `lcm(x, x) == x`, but forming `x · x` first would overflow.
    let big = 1u64 << 40;
    assert_eq!(lcm_of(big, big), Some(big));
}

#[test]
fn lcm_of_zero_is_zero() {
    // Zero is a multiple of everything.
    assert_eq!(lcm_of(0u64, 5u64), Some(0));
    assert_eq!(lcm_of(5u64, 0u64), Some(0));
    assert_eq!(lcm_of(0u64, 0u64), Some(0));
}

#[test]
fn lcm_reports_when_not_representable() {
    assert_eq!(lcm_of(u64::MAX, u64::MAX - 1), None);
    assert_eq!(lcm_of(u8::MAX, u8::MAX - 1), None);
}

#[test]
fn lcm_agrees_with_the_gcd_identity() {
    // gcd(a,b) · lcm(a,b) = a·b, on inputs small enough for the product to fit.
    for a in 1u32..=25 {
        for b in 1u32..=25 {
            let l = lcm_of(a, b).expect("small inputs are representable");
            assert_eq!(gcd_of(a, b) * l, a * b, "identity failed for {a}, {b}");
        }
    }
}
