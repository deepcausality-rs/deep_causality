/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the inherent classification predicates on `BFloat16`.

use deep_causality_num::BFloat16;

#[test]
fn test_is_nan() {
    assert!(BFloat16::NAN.is_nan());
    // Exponent all ones with any non-zero significand is a NaN, quiet bit or not.
    assert!(BFloat16::from_bits(0x7F81).is_nan());
    assert!(BFloat16::from_bits(0xFFFF).is_nan());
    assert!(!BFloat16::from_bits(0x7F80).is_nan());
    assert!(!BFloat16::ONE.is_nan());
    assert!(!BFloat16::ZERO.is_nan());
}

#[test]
fn test_is_infinite() {
    assert!(BFloat16::INFINITY.is_infinite());
    assert!(BFloat16::NEG_INFINITY.is_infinite());
    assert!(!BFloat16::NAN.is_infinite());
    assert!(!BFloat16::MAX.is_infinite());
    assert!(!BFloat16::ZERO.is_infinite());
}

#[test]
fn test_is_finite() {
    assert!(BFloat16::ONE.is_finite());
    assert!(BFloat16::MAX.is_finite());
    assert!(BFloat16::MIN.is_finite());
    assert!(BFloat16::from_bits(0x0001).is_finite());
    assert!(!BFloat16::INFINITY.is_finite());
    assert!(!BFloat16::NEG_INFINITY.is_finite());
    assert!(!BFloat16::NAN.is_finite());
}

#[test]
fn test_is_sign_positive_and_negative_read_the_sign_bit() {
    assert!(BFloat16::ONE.is_sign_positive());
    assert!(BFloat16::ZERO.is_sign_positive());
    assert!(BFloat16::INFINITY.is_sign_positive());
    assert!(BFloat16::NAN.is_sign_positive());
    assert!(!BFloat16::ONE.is_sign_negative());

    assert!(BFloat16::NEG_ZERO.is_sign_negative());
    assert!(BFloat16::MIN.is_sign_negative());
    assert!(BFloat16::NEG_INFINITY.is_sign_negative());
    assert!((-BFloat16::NAN).is_sign_negative());
    assert!(!BFloat16::NEG_ZERO.is_sign_positive());
}

#[test]
fn test_predicates_agree_with_f32_on_every_pattern() {
    for bits in 0..=u16::MAX {
        let x = BFloat16::from_bits(bits);
        let f = f32::from_bits((bits as u32) << 16);
        assert_eq!(x.is_nan(), f.is_nan(), "bits {bits:#06x}");
        assert_eq!(x.is_infinite(), f.is_infinite(), "bits {bits:#06x}");
        assert_eq!(x.is_finite(), f.is_finite(), "bits {bits:#06x}");
        assert_eq!(x.is_sign_negative(), bits & 0x8000 != 0, "bits {bits:#06x}");
        assert_eq!(x.is_sign_positive(), bits & 0x8000 == 0, "bits {bits:#06x}");
    }
}
