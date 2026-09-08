/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Arithmetic operation tests for `DoubleFloat`.

use deep_causality_num::{Float, Float106, One, Zero};

// =============================================================================
// Helper Functions
// =============================================================================

fn d(x: f64) -> Float106 {
    Float106::from(x)
}

fn approx_eq(a: Float106, b: Float106, epsilon: f64) -> bool {
    let diff = <Float106 as Float>::abs(a - b);
    diff.hi() < epsilon
}

// =============================================================================
// Basic Arithmetic Tests
// =============================================================================

#[test]
fn test_addition_basic() {
    let a = d(1.0);
    let b = d(2.0);
    let c = a + b;
    assert!(approx_eq(c, d(3.0), 1e-15));
}

#[test]
fn test_addition_with_cancellation() {
    // Test high-precision addition where f64 would lose precision
    let a = d(1.0);
    let small = Float106::new(0.0, 1e-20);
    let result = a + small;
    assert_eq!(result.hi(), 1.0);
    assert!(result.lo() > 0.0); // Small value preserved
}

#[test]
fn test_subtraction_basic() {
    let a = d(5.0);
    let b = d(3.0);
    let c = a - b;
    assert!(approx_eq(c, d(2.0), 1e-15));
}

#[test]
fn test_subtraction_self() {
    let a = d(42.0);
    let c = a - a;
    assert!(c.is_zero());
}

#[test]
fn test_multiplication_basic() {
    let a = d(3.0);
    let b = d(4.0);
    let c = a * b;
    assert!(approx_eq(c, d(12.0), 1e-15));
}

#[test]
fn test_multiplication_by_zero() {
    let a = d(42.0);
    let c = a * d(0.0);
    assert!(c.is_zero());
}

#[test]
fn test_multiplication_by_one() {
    let a = d(42.0);
    let c = a * d(1.0);
    assert!(approx_eq(c, a, 1e-15));
}

#[test]
fn test_division_basic() {
    let a = d(12.0);
    let b = d(4.0);
    let c = a / b;
    assert!(approx_eq(c, d(3.0), 1e-15));
}

#[test]
fn test_division_by_self() {
    let a = d(42.0);
    let c = a / a;
    assert!(approx_eq(c, d(1.0), 1e-15));
}

#[test]
fn test_negation() {
    let a = d(5.0);
    let b = -a;
    assert!(approx_eq(b, d(-5.0), 1e-15));
    assert!(approx_eq(-b, a, 1e-15));
}

#[test]
fn test_remainder() {
    let a = d(7.0);
    let b = d(3.0);
    let c = a % b;
    assert!(approx_eq(c, d(1.0), 1e-15));
}

// =============================================================================
// Special Values Tests
// =============================================================================

#[test]
fn test_nan_propagation() {
    let nan = <Float106 as Float>::nan();
    assert!(nan.is_nan());
    assert!((nan + d(1.0)).is_nan());
    assert!((nan * d(1.0)).is_nan());
}

#[test]
fn test_infinity() {
    let inf = <Float106 as Float>::infinity();
    assert!(inf.is_infinite());
    assert!(!inf.is_finite());
    assert!(inf > d(f64::MAX));
}

#[test]
fn test_neg_infinity() {
    let neg_inf = <Float106 as Float>::neg_infinity();
    assert!(neg_inf.is_infinite());
    assert!(neg_inf < d(f64::MIN));
}

#[test]
fn test_zero_and_one() {
    let zero = Float106::zero();
    let one = Float106::one();

    assert!(zero.is_zero());
    assert!(one.is_one());
    assert!(approx_eq(zero + one, one, 1e-15));
    assert!(approx_eq(one * one, one, 1e-15));
}

// =============================================================================
// Comparison Tests
// =============================================================================

#[test]
fn test_equality() {
    let a = d(1.0);
    let b = d(1.0);
    assert_eq!(a, b);
}

#[test]
fn test_inequality() {
    let a = d(1.0);
    let b = d(2.0);
    assert_ne!(a, b);
}

#[test]
fn test_ordering() {
    let a = d(1.0);
    let b = d(2.0);
    assert!(a < b);
    assert!(b > a);
    assert!(a <= b);
    assert!(b >= a);
}

#[test]
fn test_ordering_with_lo_component() {
    let a = Float106::new(1.0, 1e-20);
    let b = Float106::new(1.0, 2e-20);
    assert!(a < b);
}

// =============================================================================
// Assignment Operator Tests
// =============================================================================

#[test]
fn test_add_assign() {
    let mut a = d(1.0);
    a += d(2.0);
    assert!(approx_eq(a, d(3.0), 1e-15));
}

#[test]
fn test_sub_assign() {
    let mut a = d(5.0);
    a -= d(3.0);
    assert!(approx_eq(a, d(2.0), 1e-15));
}

#[test]
fn test_mul_assign() {
    let mut a = d(3.0);
    a *= d(4.0);
    assert!(approx_eq(a, d(12.0), 1e-15));
}

#[test]
fn test_div_assign() {
    let mut a = d(12.0);
    a /= d(4.0);
    assert!(approx_eq(a, d(3.0), 1e-15));
}

#[test]
fn test_rem_assign() {
    let mut a = d(7.0);
    a %= d(3.0);
    assert!(approx_eq(a, d(1.0), 1e-15));
}

// =============================================================================
// Algebraic Property Tests
// =============================================================================

#[test]
fn test_commutativity_addition() {
    let a = d(2.5);
    let b = d(3.7);
    assert_eq!(a + b, b + a);
}

#[test]
fn test_commutativity_multiplication() {
    let a = d(2.5);
    let b = d(3.7);
    assert!(approx_eq(a * b, b * a, 1e-15));
}

#[test]
fn test_associativity_addition() {
    let a = d(1.1);
    let b = d(2.2);
    let c = d(3.3);
    // Note: f64 may fail this due to rounding, but DoubleFloat should maintain it better
    let lhs = (a + b) + c;
    let rhs = a + (b + c);
    assert!(approx_eq(lhs, rhs, 1e-14));
}

#[test]
fn test_associativity_multiplication() {
    let a = d(1.1);
    let b = d(2.2);
    let c = d(3.3);
    let lhs = (a * b) * c;
    let rhs = a * (b * c);
    assert!(approx_eq(lhs, rhs, 1e-14));
}

#[test]
fn test_distributivity() {
    let a = d(2.0);
    let b = d(3.0);
    let c = d(4.0);
    let lhs = a * (b + c);
    let rhs = a * b + a * c;
    assert!(approx_eq(lhs, rhs, 1e-14));
}

// =============================================================================
// Precision Tests
// =============================================================================

#[test]
fn test_high_precision_constants() {
    let pi = Float106::PI;
    // Verify pi.hi + pi.lo is close to the true value
    let sum = pi.hi() + pi.lo();
    let expected = core::f64::consts::PI;
    assert!((sum - expected).abs() < 1e-15);
}

#[test]
fn test_precision_beyond_f64() {
    // Create a value that requires DoubleFloat precision
    let a = d(1.0);
    let tiny = Float106::new(0.0, 1e-17);
    let result = a + tiny;
    // The tiny addition should be preserved in lo component
    assert_eq!(result.hi(), 1.0);
    assert!(result.lo() > 0.0);
}

// =============================================================================
// Conversion Tests
// =============================================================================

#[test]
fn test_from_f64() {
    let x = Float106::from(42.0);
    assert_eq!(x.hi(), 42.0);
    assert_eq!(x.lo(), 0.0);
}

#[test]
fn test_to_f64() {
    let x = d(42.0);
    assert_eq!(x.to_f64(), 42.0);
}

#[test]
fn test_from_i32() {
    let x: Float106 = 42_i32.into();
    assert_eq!(x.hi(), 42.0);
}

#[test]
fn test_into_f64() {
    let x = d(42.0);
    let y: f64 = x.into();
    assert_eq!(y, 42.0);
}

// Note: Reference operation impls exist but are trivial passthrough
// to owned operations, so we don't test them separately to avoid
// clippy::op_ref warnings.

// Division follows IEEE 754 where the double-double refinement cannot go.

#[test]
fn test_division_by_zero_is_infinite_with_the_sign_of_the_dividend() {
    let q = Float106::from(1.0) / Float106::from(0.0);
    assert!(q.is_infinite() && q > Float106::from(0.0));
    let q = Float106::from(-1.0) / Float106::from(0.0);
    assert!(q.is_infinite() && q < Float106::from(0.0));
    let q = Float106::from(1.0) / 0.0;
    assert!(q.is_infinite() && q > Float106::from(0.0));
    assert!((Float106::from(0.0) / Float106::from(0.0)).is_nan());
}

#[test]
fn test_division_by_infinity_is_zero_and_of_infinity_is_infinite() {
    let inf = Float106::from(f64::INFINITY);
    assert_eq!(Float106::from(3.0) / inf, Float106::from(0.0));
    assert!((inf / Float106::from(2.0)).is_infinite());
    assert!((inf / inf).is_nan());
}

// Overflow of the high-word sum. The operands are finite, so the non-finite guard in `Add` does
// not fire, but the sum is not representable.

#[test]
fn test_addition_overflowing_finite_operands_is_infinite_not_nan() {
    // Oracle: IEEE 754-2019 §7.4. `f64::MAX + f64::MAX` overflows, and in the default
    // round-to-nearest mode an overflow delivers an infinity carrying the sign of the exact
    // result. `f64` itself is the independent implementation of that rule, and both low words are
    // zero, so the double-double sum has exactly the same exact value as the `f64` sum.
    let overflow = f64::MAX + f64::MAX;
    assert!(overflow.is_infinite() && overflow > 0.0, "f64 oracle");

    let a = d(f64::MAX);
    let b = d(f64::MAX);
    let sum = a + b;
    assert!(
        sum.is_infinite() && sum > Float106::zero(),
        "a finite pair whose sum overflows must give +inf, got hi={} lo={}",
        sum.hi(),
        sum.lo()
    );

    // The negative side, and the mixed pair whose exact sum is representable, both follow the
    // same rule: -inf, and 0 without an intermediate overflow.
    let neg = d(-f64::MAX) + d(-f64::MAX);
    assert!(
        neg.is_infinite() && neg < Float106::zero(),
        "got hi={} lo={}",
        neg.hi(),
        neg.lo()
    );
    assert_eq!((d(f64::MAX) + d(-f64::MAX)).hi(), 0.0);
}

#[test]
fn test_subtraction_overflowing_finite_operands_is_infinite_not_nan() {
    // `Sub` is `self + (-rhs)` and negation is exact, so it inherits `Add`'s guard rather than
    // carrying one. Pinned rather than assumed, and pinned on all three forms, because that
    // inheritance is the whole reason `Sub` needs no guard of its own. Same IEEE 754 §7.4 oracle.
    for (tag, diff) in [
        ("Float106 - Float106", d(f64::MAX) - d(-f64::MAX)),
        ("Float106 - f64", d(f64::MAX) - (-f64::MAX)),
        ("f64 - Float106", f64::MAX - d(-f64::MAX)),
    ] {
        assert_eq!(diff.hi(), f64::INFINITY, "{tag} high word");
        assert_eq!(diff.lo(), 0.0, "{tag} low word");
    }
    let neg = d(-f64::MAX) - d(f64::MAX);
    assert_eq!(neg.hi(), f64::NEG_INFINITY);
    assert_eq!(neg.lo(), 0.0);
}

// The same defect in the remaining error-free transforms: `two_prod`'s FMA error term is
// `a·b − inf = −inf` on an overflowing product, and the division's refinement multiplies its
// first quotient back by the divisor. Both then reach `quick_two_sum(inf, −inf)`, which is NaN.

#[test]
fn test_multiplication_overflowing_finite_operands_is_infinite_not_nan() {
    // Oracle: IEEE 754-2019 §7.4 — an overflow under round-to-nearest delivers an infinity with
    // the sign of the exact result. `f64` is the independent implementation of that rule, and
    // every operand below has a zero low word, so the double-double product has the same exact
    // value as the `f64` product.
    assert!((1e200_f64 * 1e200_f64).is_infinite(), "f64 oracle");

    for (tag, got, want) in [
        ("1e200 · 1e200", d(1e200) * d(1e200), f64::INFINITY),
        ("−1e200 · 1e200", d(-1e200) * d(1e200), f64::NEG_INFINITY),
        ("MAX · 2", d(f64::MAX) * d(2.0), f64::INFINITY),
    ] {
        assert_eq!(got.hi(), want, "{tag} high word");
        assert_eq!(got.lo(), 0.0, "{tag} low word");
    }

    // The two answers the guard must not disturb. `inf · 0` is §7.2 invalid, so NaN; and a product
    // below the range underflows to zero (§7.5) without ever reaching the guard.
    assert!((d(f64::INFINITY) * d(0.0)).is_nan());
    assert_eq!((d(1e-200) * d(1e-200)).hi(), 0.0);
}

#[test]
fn test_multiplication_by_f64_guards_non_finite_operands_and_overflow() {
    // This form had no guard at all, so it failed on a plain non-finite operand as well as on an
    // overflow: `Float106::from(2.0) * f64::INFINITY` was NaN where §7.4 and §6.1 both give +inf.
    assert!((2.0_f64 * f64::INFINITY).is_infinite(), "f64 oracle");

    for (tag, got, want) in [
        ("1e200 · 1e200", d(1e200) * 1e200_f64, f64::INFINITY),
        ("MAX · 2", d(f64::MAX) * 2.0_f64, f64::INFINITY),
        ("2 · inf", d(2.0) * f64::INFINITY, f64::INFINITY),
        ("inf · 2", d(f64::INFINITY) * 2.0_f64, f64::INFINITY),
        // `f64 * Float106` delegates here, so it is the same guard seen from the other side.
        (
            "f64 lhs: 1e200 · 1e200",
            1e200_f64 * d(1e200),
            f64::INFINITY,
        ),
    ] {
        assert_eq!(got.hi(), want, "{tag} high word");
        assert_eq!(got.lo(), 0.0, "{tag} low word");
    }

    // §7.2 invalid operations stay NaN.
    assert!((d(f64::INFINITY) * 0.0_f64).is_nan());
    assert!((d(2.0) * f64::NAN).is_nan());
}

#[test]
fn test_division_overflowing_finite_operands_is_infinite_not_nan() {
    // Oracle: IEEE 754-2019 §7.4 again, on the quotient rather than the sum or product.
    assert!((f64::MAX / 1e-200).is_infinite(), "f64 oracle");

    for (tag, got, want) in [
        ("MAX / 1e-200", d(f64::MAX) / d(1e-200), f64::INFINITY),
        ("1e300 / 1e-300", d(1e300) / d(1e-300), f64::INFINITY),
        ("−1e300 / 1e-300", d(-1e300) / d(1e-300), f64::NEG_INFINITY),
        ("f64 divisor", d(f64::MAX) / 1e-200_f64, f64::INFINITY),
        ("f64 dividend", 1e300_f64 / d(1e-300), f64::INFINITY),
    ] {
        assert_eq!(got.hi(), want, "{tag} high word");
        assert_eq!(got.lo(), 0.0, "{tag} low word");
    }

    // The guard's existing cases are unchanged: a quotient below the range underflows to zero.
    assert_eq!((d(1e-300) / d(1e300)).hi(), 0.0);
}

#[test]
fn test_remainder_of_an_out_of_range_quotient_has_no_answer() {
    // `Rem` is `a − trunc(a/b)·b`, and `trunc` has nothing to return once `a/b` leaves the range.
    // IEEE 754-2019 §5.3.1 does put a finite remainder under `|b|` on every finite pair, so NaN is
    // not the standard's answer — it is the honest report that this identity cannot reach it.
    // Carrying the quotient's infinity through would give `a − inf = −inf`, a definite and wrong
    // remainder, which is why the quotient is checked before it is used.
    //
    // Unlike its neighbours this pins an answer that did not change: the unguarded `Rem` reached
    // NaN by accident, through the NaN its unguarded `Div` handed back. It is here because fixing
    // `Div` alone turns this into `−inf`, measured.
    assert!((d(1e300) % d(1e-300)).is_nan());
    // The cases that were already NaN by the same route stay NaN: §7.2 makes both invalid.
    assert!((d(1.0) % d(0.0)).is_nan());
    assert!((d(f64::INFINITY) % d(2.0)).is_nan());
}
