/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests pinning every `BFloat16` constant to its bit pattern and its meaning.
//!
//! half-rs 1.3.1 shipped wrong values for `EPSILON`, `MAX_10_EXP`, `MAX_EXP`, `MIN_10_EXP`,
//! `MIN_EXP` and `NAN`; each of those is pinned here by a derivation, not by the constant itself.

use deep_causality_num::BFloat16;

#[test]
fn test_zero_one_and_signed_zero() {
    assert_eq!(BFloat16::ZERO.to_bits(), 0x0000);
    assert_eq!(BFloat16::NEG_ZERO.to_bits(), 0x8000);
    // 1.0 = 1.0b * 2^0: biased exponent 127 = 0x7F, so 0x7F << 7 = 0x3F80.
    assert_eq!(BFloat16::ONE.to_bits(), 0x3F80);
    assert_eq!(BFloat16::ONE.to_f32(), 1.0);
}

#[test]
fn test_epsilon_is_the_gap_above_one() {
    // Seven stored significand bits: the gap above 1 is 2^-7 = 0.0078125, biased exponent
    // 127 - 7 = 120 = 0x78, so 0x78 << 7 = 0x3C00.
    assert_eq!(BFloat16::EPSILON.to_bits(), 0x3C00);
    assert_eq!(BFloat16::EPSILON.to_f32(), 0.0078125);
    // 1 + eps is the next representable value; 1 + eps/2 ties back to 1.
    assert_eq!((BFloat16::ONE + BFloat16::EPSILON).to_bits(), 0x3F81);
    let half_eps = BFloat16::from_bits(0x3B80);
    assert_eq!(half_eps.to_f32(), 0.00390625);
    assert_eq!(BFloat16::ONE + half_eps, BFloat16::ONE);
}

#[test]
fn test_range_constants() {
    // MAX has the largest finite exponent 254 = 0xFE and a full significand: (0xFE << 7) | 0x7F.
    assert_eq!(BFloat16::MAX.to_bits(), 0x7F7F);
    assert_eq!(BFloat16::MAX.to_f32().to_bits(), 0x7F7F_0000);
    assert_eq!(BFloat16::MIN.to_bits(), 0xFF7F);
    assert_eq!(BFloat16::MIN, -BFloat16::MAX);
    // The smallest normal has biased exponent 1: 1 << 7 = 0x0080 = 2^-126 = f32::MIN_POSITIVE.
    assert_eq!(BFloat16::MIN_POSITIVE.to_bits(), 0x0080);
    assert_eq!(BFloat16::MIN_POSITIVE.to_f32(), f32::MIN_POSITIVE);
    // Anything at or above MAX + half an ulp is infinity; EPSILON * MAX is nearly two ulps.
    assert_eq!(
        (BFloat16::MAX + BFloat16::EPSILON * BFloat16::MAX).to_bits(),
        0x7F80
    );
}

#[test]
fn test_non_finite_constants() {
    // Exponent all ones, significand zero: 0xFF << 7 = 0x7F80.
    assert_eq!(BFloat16::INFINITY.to_bits(), 0x7F80);
    assert_eq!(BFloat16::NEG_INFINITY.to_bits(), 0xFF80);
    // The canonical quiet NaN sets the top significand bit: 0x7F80 | 0x0040, matching the top
    // half of f32::NAN = 0x7FC00000.
    assert_eq!(BFloat16::NAN.to_bits(), 0x7FC0);
    assert!(BFloat16::NAN.is_nan());
    assert_eq!(BFloat16::NAN.to_f32().to_bits(), f32::NAN.to_bits());
    assert_eq!(BFloat16::INFINITY.to_f32(), f32::INFINITY);
    assert_eq!(BFloat16::NEG_INFINITY.to_f32(), f32::NEG_INFINITY);
}

#[test]
fn test_mathematical_constants_are_correctly_rounded() {
    // pi = 3.14159...; in [2, 4) the step is 2^-6 and (pi - 2) / 2^-6 = 73.06, so 2 + 73/64 =
    // 3.140625 with significand 73 = 0x49 and biased exponent 128 = 0x80: 0x4049.
    assert_eq!(BFloat16::PI.to_bits(), 0x4049);
    assert_eq!(BFloat16::PI.to_f32(), 3.140625);
    // e = 2.71828...; (e - 2) / 2^-6 = 45.97, so 2 + 46/64 = 2.71875, significand 0x2E: 0x402E.
    assert_eq!(BFloat16::E.to_bits(), 0x402E);
    assert_eq!(BFloat16::E.to_f32(), 2.71875);
    // ln 2 = 0.693147...; in [0.5, 1) the step is 2^-8 and (ln2 - 0.5) / 2^-8 = 49.45, so
    // 0.5 + 49/256 = 0.69140625, significand 0x31, biased exponent 126 = 0x7E: 0x3F31.
    assert_eq!(BFloat16::LN_2.to_bits(), 0x3F31);
    assert_eq!(BFloat16::LN_2.to_f32(), 0.69140625);
    // ln 10 = 2.302585...; (ln10 - 2) / 2^-6 = 19.37, so 2 + 19/64 = 2.296875: 0x4013.
    assert_eq!(BFloat16::LN_10.to_bits(), 0x4013);
    assert_eq!(BFloat16::LN_10.to_f32(), 2.296875);
}

#[test]
fn test_mathematical_constants_are_the_nearest_representable_values() {
    // Each constant is closer to the true value than either of its neighbours.
    for (c, exact) in [
        (BFloat16::PI, core::f64::consts::PI),
        (BFloat16::E, core::f64::consts::E),
        (BFloat16::LN_2, core::f64::consts::LN_2),
        (BFloat16::LN_10, core::f64::consts::LN_10),
    ] {
        let below = BFloat16::from_bits(c.to_bits() - 1).to_f64();
        let above = BFloat16::from_bits(c.to_bits() + 1).to_f64();
        let err = (c.to_f64() - exact).abs();
        assert!(err < (below - exact).abs(), "{exact}");
        assert!(err < (above - exact).abs(), "{exact}");
    }
}

#[test]
fn test_format_descriptors() {
    assert_eq!(BFloat16::RADIX, 2);
    // Seven stored bits plus the implicit one.
    assert_eq!(BFloat16::MANTISSA_DIGITS, 8);
    // floor((8 - 1) * log10(2)) = floor(2.107) = 2 decimal digits survive a round trip.
    assert_eq!(BFloat16::DIGITS, 2);
    // The exponent field is that of binary32: normals span 2^-126 ..= (2 - 2^-7) * 2^127, which
    // in the std convention (value = significand * 2^exp, significand in [0.5, 1)) is
    // MIN_EXP = -125 and MAX_EXP = 128.
    assert_eq!(BFloat16::MIN_EXP, -125);
    assert_eq!(BFloat16::MAX_EXP, 128);
    // 10^-37 lies above 2^-126 = 1.18e-38 and 10^38 below MAX = 3.39e38.
    assert_eq!(BFloat16::MIN_10_EXP, -37);
    assert_eq!(BFloat16::MAX_10_EXP, 38);
    // The descriptors agree with the range constants.
    assert_eq!(
        2f64.powi(BFloat16::MIN_EXP - 1),
        BFloat16::MIN_POSITIVE.to_f64()
    );
    assert!(BFloat16::MAX.to_f64() < 2f64.powi(BFloat16::MAX_EXP));
    assert!(BFloat16::MAX.to_f64() > 2f64.powi(BFloat16::MAX_EXP - 1));
    assert!(10f64.powi(BFloat16::MAX_10_EXP) < BFloat16::MAX.to_f64());
    assert!(10f64.powi(BFloat16::MAX_10_EXP + 1) > BFloat16::MAX.to_f64());
    assert!(10f64.powi(BFloat16::MIN_10_EXP) > BFloat16::MIN_POSITIVE.to_f64());
    assert!(10f64.powi(BFloat16::MIN_10_EXP - 1) < BFloat16::MIN_POSITIVE.to_f64());
}
