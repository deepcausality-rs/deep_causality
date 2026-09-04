/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the `From` conversions: rounding on the way in, exact on the way out.

use deep_causality_num::BFloat16;

// =============================================================================
// From<f32> and From<f64> round to nearest even
// =============================================================================

#[test]
fn test_from_f32_rounds() {
    // 0.1f32 = 0x3DCCCCCD, low half above the midpoint: 0x3DCD.
    let x: BFloat16 = 0.1_f32.into();
    assert_eq!(x.to_bits(), 0x3DCD);
    // 1.5 = 1.1b: significand 0x40 on exponent 0x7F, i.e. 0x3FC0.
    assert_eq!(BFloat16::from(1.5_f32).to_bits(), 0x3FC0);
    // The tie 1 + 2^-8 goes to the even neighbour 1.0.
    assert_eq!(
        BFloat16::from(f32::from_bits(0x3F80_8000)).to_bits(),
        0x3F80
    );
}

#[test]
fn test_from_f64_rounds_once() {
    let x: BFloat16 = 0.1_f64.into();
    assert_eq!(x.to_bits(), 0x3DCD);
    // -42.5 = -1.0101010b * 2^5: sign 1, biased exponent 132 = 0x84, significand 0x2A: 0xC22A.
    assert_eq!(BFloat16::from(-42.5_f64).to_bits(), 0xC22A);
    // Just above the tie at 1 + 2^-8: the value above it, not the tie's even neighbour.
    let above_tie = 1.0 + 2f64.powi(-8) + 2f64.powi(-30);
    assert_eq!(BFloat16::from(above_tie).to_bits(), 0x3F81);
}

#[test]
fn test_from_f32_and_from_f64_agree_on_exact_inputs() {
    for v in [0.0, -0.0, 1.0, -1.0, 0.375, 1000.0, 3.140625, 1e-40] {
        assert_eq!(BFloat16::from(v as f32), BFloat16::from(v), "{v}");
    }
}

// =============================================================================
// From<BFloat16> for f32 and f64 are exact
// =============================================================================

#[test]
fn test_into_f32_is_exact() {
    let x = BFloat16::from_bits(0x3EAB);
    let y: f32 = x.into();
    // 0x3EAB = 171/512; the quotient is exact in f32.
    assert_eq!(y, 171.0 / 512.0);
    assert_eq!(f32::from(BFloat16::MAX).to_bits(), 0x7F7F_0000);
    assert_eq!(f32::from(BFloat16::NEG_ZERO).to_bits(), 0x8000_0000);
}

#[test]
fn test_into_f64_is_exact() {
    let x = BFloat16::from_bits(0x3DCD);
    let y: f64 = x.into();
    assert_eq!(y, 0.10009765625);
    assert_eq!(f64::from(BFloat16::from_bits(0x0001)), 2f64.powi(-133));
    assert!(f64::from(BFloat16::NAN).is_nan());
}

#[test]
fn test_round_trip_through_f32_and_f64_is_the_identity() {
    for bits in [
        0x0000, 0x8000, 0x3F80, 0xBF80, 0x0001, 0x007F, 0x0080, 0x7F7F, 0xFF7F, 0x7F80, 0xFF80,
    ] {
        let x = BFloat16::from_bits(bits);
        assert_eq!(BFloat16::from(f32::from(x)).to_bits(), bits, "{bits:#06x}");
        assert_eq!(BFloat16::from(f64::from(x)).to_bits(), bits, "{bits:#06x}");
    }
}
