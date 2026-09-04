/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The blanket implementations `BFloat16` reaches through `RealField` that carry a body rather
//! than only a marker: `Normed`, `ConjugateScalar`, `InvMonoid` and `NormedScalar`.
//!
//! The tower's marker aggregations — `Ring`, `Field`, `AbelianGroup` and the rest — are empty
//! blankets, and `bfloat16_traits_tests` pins them by compiling the bound. These four compute
//! something, so reaching the bound is not the whole claim; what they compute at eight
//! significand bits is tested here.

use deep_causality_algebra::{ConjugateScalar, InvMonoid, Normed, NormedScalar, Real};
use deep_causality_num::BFloat16;

fn bf(x: f32) -> BFloat16 {
    let v = BFloat16::from(x);
    assert_eq!(
        v.to_f32(),
        x,
        "test operand {x} is not exactly representable"
    );
    v
}

fn assert_normed_scalar<T: NormedScalar>() {}

#[test]
fn test_normed_scalar_bound() {
    assert_normed_scalar::<BFloat16>();
}

// =============================================================================
// Normed
// =============================================================================

#[test]
fn test_modulus_squared_and_scale_by_real() {
    assert_eq!(Normed::modulus_squared(&bf(3.0)), bf(9.0));
    assert_eq!(Normed::modulus_squared(&bf(-3.0)), bf(9.0));
    assert_eq!(Normed::modulus_squared(&BFloat16::ZERO), BFloat16::ZERO);
    assert_eq!(Normed::scale_by_real(&bf(3.0), bf(4.0)), bf(12.0));
    assert_eq!(Normed::scale_by_real(&bf(-3.0), bf(4.0)), bf(-12.0));
    // Scaling by zero keeps the sign of the product, so -3 * 0 is -0.
    assert_eq!(
        Normed::scale_by_real(&bf(-3.0), BFloat16::ZERO).to_bits(),
        0x8000
    );
}

#[test]
fn test_modulus_is_exact_where_the_squared_modulus_overflows() {
    // `modulus` is `abs`, not `modulus_squared().sqrt()`. The two part company at the top of the
    // range: MAX^2 is beyond the format, so a sqrt-of-square modulus would report infinity for a
    // value the type represents exactly. bf16 has f32's exponent range, so the square of anything
    // above about 1.8e19 overflows.
    assert!(Normed::modulus_squared(&BFloat16::MAX).is_infinite());
    assert_eq!(Normed::modulus(&BFloat16::MAX), BFloat16::MAX);
    assert_eq!(Normed::modulus(&BFloat16::MIN), BFloat16::MAX);

    // The same parting at the bottom: MIN_POSITIVE^2 underflows to zero, but the magnitude is the
    // value itself.
    assert_eq!(
        Normed::modulus_squared(&BFloat16::MIN_POSITIVE),
        BFloat16::ZERO
    );
    assert_eq!(
        Normed::modulus(&BFloat16::MIN_POSITIVE),
        BFloat16::MIN_POSITIVE
    );
}

#[test]
fn test_modulus_is_abs_on_every_pattern() {
    // Exhaustive over the format: 65536 patterns, every one of which is a bf16.
    for bits in 0u16..=u16::MAX {
        let x = BFloat16::from_bits(bits);
        if Real::is_nan(x) {
            assert!(Real::is_nan(Normed::modulus(&x)), "{bits:#06x}");
            continue;
        }
        assert_eq!(Normed::modulus(&x), Real::abs(x), "{bits:#06x}");
        // A magnitude is never negative, and the sign bit is the whole difference.
        assert_eq!(Normed::modulus(&x).to_bits(), bits & 0x7FFF, "{bits:#06x}");
    }
}

// =============================================================================
// ConjugateScalar
// =============================================================================

#[test]
fn test_conjugation_is_the_identity_on_every_pattern() {
    for bits in 0u16..=u16::MAX {
        let x = BFloat16::from_bits(bits);
        // Bit equality, so the signed zeros and every NaN payload are covered too.
        assert_eq!(
            ConjugateScalar::conjugate(&x).to_bits(),
            bits,
            "{bits:#06x}"
        );
        assert_eq!(
            ConjugateScalar::real_part(&x).to_bits(),
            bits,
            "{bits:#06x}"
        );
        assert_eq!(
            <BFloat16 as ConjugateScalar>::from_real(x).to_bits(),
            bits,
            "{bits:#06x}"
        );
    }
}

#[test]
fn test_conjugate_scalar_modulus_matches_the_normed_one() {
    assert_eq!(ConjugateScalar::modulus_squared(&bf(-3.0)), bf(9.0));
    assert_eq!(ConjugateScalar::modulus(&bf(-3.0)), bf(3.0));
    // The same overflow parting as `Normed`: this modulus is `abs` as well.
    assert!(ConjugateScalar::modulus_squared(&BFloat16::MAX).is_infinite());
    assert_eq!(ConjugateScalar::modulus(&BFloat16::MIN), BFloat16::MAX);
}

// =============================================================================
// InvMonoid
// =============================================================================

#[test]
fn test_inverse_is_one_over_the_value() {
    // A power of two inverts exactly at any precision.
    assert_eq!(InvMonoid::inverse(&bf(4.0)), bf(0.25));
    assert_eq!(InvMonoid::inverse(&bf(-4.0)), bf(-0.25));
    assert_eq!(InvMonoid::inverse(&BFloat16::ONE), BFloat16::ONE);

    // 1/3 is not representable: the blanket divides and rounds once, so the answer is the
    // correctly rounded 0.333..., which is 0x3EAB.
    assert_eq!(InvMonoid::inverse(&bf(3.0)).to_bits(), 0x3EAB);

    // The multiplicative identity holds wherever the inverse is exact.
    for k in 0..8u32 {
        let x = bf(f32::powi(2.0, k as i32));
        assert_eq!(x * InvMonoid::inverse(&x), BFloat16::ONE, "2^{k}");
    }
}

#[test]
fn test_inverse_at_zero_and_the_non_finite_values() {
    assert_eq!(InvMonoid::inverse(&BFloat16::ZERO), BFloat16::INFINITY);
    assert_eq!(
        InvMonoid::inverse(&BFloat16::NEG_ZERO),
        BFloat16::NEG_INFINITY
    );
    assert_eq!(InvMonoid::inverse(&BFloat16::INFINITY), BFloat16::ZERO);
    assert!(Real::is_nan(InvMonoid::inverse(&BFloat16::NAN)));
}
