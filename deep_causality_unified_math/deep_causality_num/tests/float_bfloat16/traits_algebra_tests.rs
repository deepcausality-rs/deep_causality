/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `Zero`, `One` and `Num` on `BFloat16`.

use deep_causality_num::{BFloat16, Num, One, Zero};

// =============================================================================
// Zero
// =============================================================================

#[test]
fn test_zero() {
    let zero = BFloat16::zero();
    assert_eq!(zero.to_bits(), 0x0000);
    assert!(zero.is_zero());
    assert!(!BFloat16::ONE.is_zero());
    assert!(!BFloat16::from_bits(0x0001).is_zero());
    assert!(!BFloat16::NAN.is_zero());
}

#[test]
fn test_negative_zero_is_zero() {
    // A bit-pattern comparison against 0x0000 would miss this.
    assert!(BFloat16::NEG_ZERO.is_zero());
}

#[test]
fn test_set_zero() {
    let mut x = BFloat16::from(42.0);
    x.set_zero();
    assert!(x.is_zero());
    assert_eq!(x.to_bits(), 0x0000);
}

#[test]
fn test_zero_is_the_additive_identity() {
    for bits in [0x3F80, 0xBF80, 0x0001, 0x7F7F, 0x4049, 0xFF80] {
        let x = BFloat16::from_bits(bits);
        assert_eq!(x + BFloat16::zero(), x, "{bits:#06x}");
        assert_eq!(BFloat16::zero() + x, x, "{bits:#06x}");
    }
}

// =============================================================================
// One
// =============================================================================

#[test]
fn test_one() {
    let one = BFloat16::one();
    assert_eq!(one.to_bits(), 0x3F80);
    assert!(one.is_one());
    assert!(!BFloat16::ZERO.is_one());
    assert!(!BFloat16::from_bits(0x3F81).is_one());
    assert!(!BFloat16::from_bits(0xBF80).is_one());
    assert!(!BFloat16::NAN.is_one());
}

#[test]
fn test_set_one() {
    let mut x = BFloat16::from(42.0);
    x.set_one();
    assert!(x.is_one());
}

#[test]
fn test_one_is_the_multiplicative_identity() {
    for bits in [0x3F80, 0xBF80, 0x0001, 0x7F7F, 0x4049, 0xFF80] {
        let x = BFloat16::from_bits(bits);
        assert_eq!(x * BFloat16::one(), x, "{bits:#06x}");
        assert_eq!(BFloat16::one() * x, x, "{bits:#06x}");
    }
}

// =============================================================================
// Num
// =============================================================================

fn assert_num<T: Num>() {}

#[test]
fn test_num_bound() {
    assert_num::<BFloat16>();
}
