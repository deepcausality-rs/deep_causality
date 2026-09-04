/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the widening getters `to_f32` and `to_f64`, which are exact.

use deep_causality_num::BFloat16;

#[test]
fn test_to_f32_is_the_top_half_of_the_f32_pattern() {
    assert_eq!(BFloat16::from_bits(0x3F80).to_f32().to_bits(), 0x3F80_0000);
    assert_eq!(BFloat16::from_bits(0x0001).to_f32().to_bits(), 0x0001_0000);
    assert_eq!(BFloat16::from_bits(0xC000).to_f32(), -2.0);
    // 0x3EAB = 1.0101011b * 2^-2 = 171/512 = 0.333984375; the quotient is exact in f32.
    assert_eq!(BFloat16::from_bits(0x3EAB).to_f32(), 171.0 / 512.0);
}

#[test]
fn test_to_f64_is_exact() {
    assert_eq!(BFloat16::from_bits(0x3EAB).to_f64(), 0.333984375);
    // 0x3DCD = 1.1001101b * 2^-4 = 205/128 / 16 = 0.10009765625.
    assert_eq!(BFloat16::from_bits(0x3DCD).to_f64(), 0.10009765625);
    // MAX = (2 - 2^-7) * 2^127 = 3.3895313892515355e38.
    assert_eq!(BFloat16::from_bits(0x7F7F).to_f64(), 3.3895313892515355e38);
    assert_eq!(BFloat16::from_bits(0x0001).to_f64(), 2f64.powi(-133));
}

#[test]
fn test_getters_preserve_special_values() {
    assert!(BFloat16::from_bits(0x7FC0).to_f32().is_nan());
    assert!(BFloat16::from_bits(0x7FC0).to_f64().is_nan());
    assert_eq!(BFloat16::from_bits(0x7F80).to_f32(), f32::INFINITY);
    assert_eq!(BFloat16::from_bits(0xFF80).to_f64(), f64::NEG_INFINITY);
    assert!(BFloat16::from_bits(0x8000).to_f32().is_sign_negative());
    assert_eq!(BFloat16::from_bits(0x8000).to_f64(), 0.0);
}

#[test]
fn test_to_f32_then_to_f64_matches_to_f64_everywhere() {
    for bits in 0..=u16::MAX {
        let x = BFloat16::from_bits(bits);
        if x.is_nan() {
            continue;
        }
        assert_eq!(x.to_f32() as f64, x.to_f64(), "bits {bits:#06x}");
    }
}
