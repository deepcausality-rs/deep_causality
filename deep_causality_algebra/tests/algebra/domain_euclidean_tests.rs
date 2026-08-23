/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `EuclideanDomain`, the level of the tower at which exact integer arithmetic lives.
//!
//! These exercise the trait through generic bounds rather than through the inherent integer
//! methods, so what is under test is the abstraction, not `i64`'s own `div_euclid`. The
//! inherent methods take their operand by value and win method resolution, so the trait calls
//! are written in fully-qualified form.

use deep_causality_algebra::EuclideanDomain;

/// Generic over the abstraction, never over a concrete width.
fn gcd_of<T: EuclideanDomain + Clone>(a: T, b: T) -> T {
    a.gcd(&b)
}

/// No `Div` bound. This signature is the regression test for `lcm`: the default body can only
/// use what these bounds permit, so it compiles only while `lcm` takes its quotient with
/// `div_euclid` rather than with the `/` operator. Re-introducing `/` breaks this file.
fn lcm_of<T: EuclideanDomain + Clone>(a: T, b: T) -> T {
    a.lcm(&b)
}

#[test]
fn gcd_is_computed_through_the_trait() {
    assert_eq!(gcd_of(48i64, 18i64), 6);
    assert_eq!(gcd_of(17i32, 5i32), 1);
    assert_eq!(gcd_of(270i128, 192i128), 6);
    assert_eq!(gcd_of(13i16, 13i16), 13);
}

#[test]
fn gcd_with_zero_returns_the_other_operand() {
    assert_eq!(gcd_of(0i64, 7i64), 7);
    assert_eq!(gcd_of(7i64, 0i64), 7);
    assert_eq!(gcd_of(0i64, 0i64), 0);
}

#[test]
fn gcd_is_non_negative_for_negative_inputs() {
    // `rem_euclid` keeps the remainder non-negative, so the algorithm terminates on a
    // non-negative gcd even when an operand is negative.
    assert_eq!(gcd_of(-48i64, 18i64), 6);
    assert_eq!(gcd_of(48i64, -18i64), 6);
    assert_eq!(gcd_of(-48i64, -18i64), 6);
}

#[test]
fn lcm_is_computed_through_the_trait() {
    assert_eq!(lcm_of(4i64, 6i64), 12);
    assert_eq!(lcm_of(21i32, 6i32), 42);
}

#[test]
fn lcm_of_zero_is_zero() {
    assert_eq!(lcm_of(0i64, 5i64), 0);
    assert_eq!(lcm_of(5i64, 0i64), 0);
}

#[test]
fn division_law_holds_over_a_range() {
    // The defining axiom: a = b·q + r with 0 <= r < |b|.
    for a in -50i64..=50 {
        for b in -9i64..=9 {
            if b == 0 {
                continue;
            }
            let q = EuclideanDomain::div_euclid(&a, &b);
            let r = EuclideanDomain::rem_euclid(&a, &b);
            assert_eq!(a, b * q + r, "reconstruction failed for a={a}, b={b}");
            assert!(
                r >= 0,
                "remainder must be non-negative: a={a}, b={b}, r={r}"
            );
            assert!(
                (r as u64) < b.unsigned_abs(),
                "remainder must be smaller than |b|: a={a}, b={b}, r={r}"
            );
        }
    }
}

#[test]
fn euclidean_fn_is_the_absolute_value() {
    assert_eq!(EuclideanDomain::euclidean_fn(&-5i32), 5u32);
    assert_eq!(EuclideanDomain::euclidean_fn(&5i32), 5u32);
    assert_eq!(EuclideanDomain::euclidean_fn(&0i64), 0u64);
}

#[test]
fn euclidean_fn_is_unsigned_so_min_is_representable() {
    // This is why `EuclideanValue` is the unsigned counterpart rather than `Self`:
    // |i32::MIN| does not fit in an i32.
    assert_eq!(EuclideanDomain::euclidean_fn(&i32::MIN), 2_147_483_648u32);
    assert_eq!(
        EuclideanDomain::euclidean_fn(&i64::MIN),
        9_223_372_036_854_775_808u64
    );
}

#[test]
fn euclidean_fn_strictly_decreases() {
    // The termination guarantee: φ(r) < φ(b) whenever r != 0.
    let (a, b) = (1071i64, 462i64);
    let r = EuclideanDomain::rem_euclid(&a, &b);
    assert_ne!(r, 0);
    assert!(EuclideanDomain::euclidean_fn(&r) < EuclideanDomain::euclidean_fn(&b));
}

#[test]
fn every_signed_width_is_a_euclidean_domain() {
    fn assert_euclidean<T: EuclideanDomain>() {}
    assert_euclidean::<i8>();
    assert_euclidean::<i16>();
    assert_euclidean::<i32>();
    assert_euclidean::<i64>();
    assert_euclidean::<i128>();
    assert_euclidean::<isize>();
}

#[test]
fn lcm_is_non_negative() {
    // The doc promises |a·b| / gcd, so the result is normalized like `gcd` is.
    assert_eq!(lcm_of(-4i64, 6i64), 12);
    assert_eq!(lcm_of(4i64, -6i64), 12);
    assert_eq!(lcm_of(-4i64, -6i64), 12);
}

#[test]
fn lcm_divides_before_multiplying() {
    // `lcm(x, x) == x`, but forming `x · x` first overflows: 2⁴⁰ · 2⁴⁰ = 2⁸⁰ exceeds i64.
    // Dividing by the gcd first keeps the intermediate no larger than the answer.
    let big = 1i64 << 40;
    assert_eq!(lcm_of(big, big), big);
    // A case where the answer fits but the naive product would not.
    let a = 1i64 << 40;
    let b = 3i64 << 40;
    assert_eq!(lcm_of(a, b), b);
}

#[test]
fn lcm_agrees_with_the_gcd_identity() {
    // gcd(a,b) · lcm(a,b) = |a·b| on inputs small enough for the product to fit.
    for a in 1i64..=20 {
        for b in 1i64..=20 {
            assert_eq!(a.gcd(&b) * lcm_of(a, b), a * b, "failed for a={a}, b={b}");
        }
    }
}

// -----------------------------------------------------------------------------
// The signed minimum: where `normalize` and `gcd` run out of representation.
// -----------------------------------------------------------------------------

#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "attempt to negate with overflow")]
fn normalize_panics_at_the_signed_minimum_in_debug() {
    // Documented partiality: |T::MIN| is one past the top of the type, so the canonical
    // associate does not exist. Debug builds trap on the negation.
    let _ = EuclideanDomain::normalize(&core::hint::black_box(i64::MIN));
}

#[test]
#[cfg(not(debug_assertions))]
fn normalize_wraps_at_the_signed_minimum_in_release() {
    // The same input in release wraps back to a *negative* value, which is exactly why the
    // non-negativity guarantee is documented as partial rather than absolute.
    assert_eq!(
        EuclideanDomain::normalize(&core::hint::black_box(i64::MIN)),
        i64::MIN
    );
}

#[test]
fn checked_normalize_reports_the_signed_minimum() {
    // Same function, total: the one input with no canonical associate comes back as `None`.
    fn assert_min_has_no_associate<T>(min: T)
    where
        T: EuclideanDomain + Copy + core::fmt::Debug + PartialEq,
    {
        assert_eq!(min.checked_normalize(), None, "expected None at {min:?}");
    }

    assert_min_has_no_associate(i8::MIN);
    assert_min_has_no_associate(i16::MIN);
    assert_min_has_no_associate(i32::MIN);
    assert_min_has_no_associate(i64::MIN);
    assert_min_has_no_associate(i128::MIN);
    assert_min_has_no_associate(isize::MIN);
}

#[test]
fn checked_normalize_agrees_with_normalize_everywhere_else() {
    for n in -128i64..=128 {
        assert_eq!(n.checked_normalize(), Some(EuclideanDomain::normalize(&n)));
    }
    assert_eq!(i64::MAX.checked_normalize(), Some(i64::MAX));
    assert_eq!((i64::MIN + 1).checked_normalize(), Some(i64::MAX));
}

#[test]
fn checked_gcd_reports_an_unrepresentable_result() {
    // gcd(MIN, 0) is |MIN|, which the type cannot hold. The total `gcd` cannot honour its
    // non-negativity contract here; `checked_gcd` says so instead.
    assert_eq!(EuclideanDomain::checked_gcd(&i64::MIN, &0i64), None);
    assert_eq!(EuclideanDomain::checked_gcd(&i64::MIN, &i64::MIN), None);
    assert_eq!(EuclideanDomain::checked_gcd(&i8::MIN, &0i8), None);
}

#[test]
fn checked_gcd_agrees_with_gcd_where_the_result_is_representable() {
    // MIN is 2^63, so its gcd with 6 is 2 — representable, and returned.
    assert_eq!(EuclideanDomain::checked_gcd(&i64::MIN, &6i64), Some(2));
    for a in -24i64..=24 {
        for b in -12i64..=12 {
            assert_eq!(EuclideanDomain::checked_gcd(&a, &b), Some(a.gcd(&b)));
        }
    }
}
