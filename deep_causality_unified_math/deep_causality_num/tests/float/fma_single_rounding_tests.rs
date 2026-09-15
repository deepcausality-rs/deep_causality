/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Every `Float` implementation fuses its multiply-add.
//!
//! [`Float::mul_add`] promises `(self * a) + b` "with only one rounding error". That promise is
//! stated on the trait, so it binds all four implementations, and this file checks them together.
//!
//! The property is one sentence: **there is a product whose exact value needs more room than the
//! type has, and an addend that cancels everything the type kept.** Round the product on its own
//! and the answer is gone before the addend arrives; carry the product in whole and it survives.
//!
//! Finding such a case takes two different probes, because the four types are shaped two different
//! ways, and that difference is worth stating.

use deep_causality_num::{BFloat16, Float, Float106};

// =============================================================================
// The three formats with a uniform significand
// =============================================================================

/// The smallest power of two whose square is far enough out that `s² − 1` rounds back to `s²`.
///
/// Doubling from two keeps `s` a power of two, so `s²` stays exactly representable however far the
/// search runs, and the loop stops as soon as the type can no longer see the `− 1`.
///
/// Returns `None` for a type that never stops seeing it. A format with `p` significand bits always
/// stops, at `s² > 2^p`; [`Float106`] never does, for the reason given below.
fn significand_edge<T: Float>() -> Option<T> {
    let two = T::one() + T::one();
    let mut s = two;

    while (s * s).is_finite() {
        if s * s - T::one() == s * s {
            return Some(s);
        }
        s *= two;
    }

    None
}

/// `(s+1)(s−1) − s²` is `−1` for every `s`. At the significand edge the `−1` sits below the last
/// bit of the product, so only a fused multiply-add returns it.
fn assert_fuses_at_the_edge<T: Float + core::fmt::Debug>(name: &str) {
    let s = significand_edge::<T>().expect("a uniform significand has an edge");
    let one = T::one();

    let unfused = (s + one) * (s - one) - s * s;
    let fused = Float::mul_add(s + one, s - one, -(s * s));

    assert_eq!(
        unfused,
        T::zero(),
        "{name}: the unfused form should lose the -1, so the probe is not measuring what it claims"
    );
    assert_eq!(fused, -one, "{name}: mul_add rounded more than once");
}

#[test]
fn test_bfloat16_mul_add_rounds_once() {
    assert_fuses_at_the_edge::<BFloat16>("BFloat16");
}

#[test]
fn test_f32_mul_add_rounds_once() {
    assert_fuses_at_the_edge::<f32>("f32");
}

#[test]
fn test_f64_mul_add_rounds_once() {
    assert_fuses_at_the_edge::<f64>("f64");
}

// =============================================================================
// The double-double, which is shaped differently
// =============================================================================

/// `Float106` has no significand edge, so the probe above never terminates on it.
///
/// A double-double is two `f64` limbs held as an unevaluated sum, and its significand is not a
/// fixed width: `2^k − 1` is exactly representable at every scale, as `hi = 2^k` beside
/// `lo = −1`. The high word carries the magnitude and the low word carries whatever is left, so
/// there is no point at which subtracting one stops being visible.
#[test]
fn test_float106_has_no_significand_edge() {
    assert!(
        significand_edge::<Float106>().is_none(),
        "a double-double represents 2^k - 1 at every scale, so the search has no stopping point"
    );
}

/// What `Float106` drops instead is the `lo · lo` term of the product.
///
/// `Mul` keeps the exact leading product and both large cross terms, and discards `self.lo * a.lo`
/// as `O(ulp²)`. That is the right call for a product standing alone. Here `b` cancels everything
/// above it, so `lo · lo` is the whole answer, and a `mul_add` that rounds the product first
/// returns zero.
#[test]
fn test_float106_mul_add_rounds_once() {
    let one = Float106::from(1.0);

    for exponent in [54, 55, 60, 80] {
        let s = Float106::from(2f64.powi(exponent));

        let unfused = (s + one) * (s - one) - s * s;
        let fused = Float::mul_add(s + one, s - one, -(s * s));

        assert_eq!(
            unfused,
            Float106::from(0.0),
            "2^{exponent}: the unfused form should lose the -1"
        );
        assert_eq!(fused, -one, "2^{exponent}: mul_add rounded more than once");
    }
}
