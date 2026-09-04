/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `PartialEq` and `PartialOrd` on `BFloat16`, which follow IEEE 754 comparison.

use core::cmp::Ordering;
use deep_causality_num::BFloat16;

#[test]
fn test_equality_of_ordinary_values() {
    assert_eq!(BFloat16::from(1.5), BFloat16::from(1.5));
    assert_ne!(BFloat16::from(1.5), BFloat16::from(2.5));
    assert_eq!(BFloat16::from_bits(0x3F80), BFloat16::ONE);
}

#[test]
fn test_signed_zeros_compare_equal() {
    // half-rs 1.0.2 fixed positive/negative zero comparison; +0 and -0 differ in bits and are
    // equal as values.
    assert_ne!(BFloat16::ZERO.to_bits(), BFloat16::NEG_ZERO.to_bits());
    assert_eq!(BFloat16::ZERO, BFloat16::NEG_ZERO);
    assert_eq!(
        BFloat16::ZERO.partial_cmp(&BFloat16::NEG_ZERO),
        Some(Ordering::Equal)
    );
    assert!(BFloat16::ZERO <= BFloat16::NEG_ZERO);
    assert!(BFloat16::ZERO >= BFloat16::NEG_ZERO);
    assert!(!BFloat16::ZERO.lt(&BFloat16::NEG_ZERO));
    assert!(!BFloat16::NEG_ZERO.lt(&BFloat16::ZERO));
}

#[test]
fn test_nan_is_unordered_and_unequal_to_everything() {
    let nan = BFloat16::NAN;
    assert_ne!(nan, nan);
    // Every ordering predicate is false against a NaN; the method forms make explicit that these
    // are not the negations of one another, which is the point of a partial order.
    assert!(!nan.lt(&BFloat16::ONE));
    assert!(!nan.gt(&BFloat16::ONE));
    assert!(!nan.le(&BFloat16::ONE));
    assert!(!nan.ge(&BFloat16::ONE));
    assert!(!BFloat16::ONE.lt(&nan));
    assert!(!BFloat16::ONE.ge(&nan));
    assert_eq!(nan.partial_cmp(&BFloat16::ONE), None);
    assert_eq!(BFloat16::ONE.partial_cmp(&nan), None);
    assert_eq!(nan.partial_cmp(&nan), None);
    // Two NaNs with the same bits are still unequal.
    assert_ne!(BFloat16::from_bits(0x7FC1), BFloat16::from_bits(0x7FC1));
}

#[test]
fn test_ordering() {
    assert!(BFloat16::ONE < BFloat16::from(2.0));
    assert!(BFloat16::from(-1.0) < BFloat16::ZERO);
    assert!(BFloat16::from(-2.0) < BFloat16::from(-1.0));
    assert!(BFloat16::NEG_INFINITY < BFloat16::MIN);
    assert!(BFloat16::MAX < BFloat16::INFINITY);
    assert!(BFloat16::from_bits(0x0001) < BFloat16::MIN_POSITIVE);
    assert!(BFloat16::ZERO < BFloat16::from_bits(0x0001));
    assert!(BFloat16::from_bits(0x8001) < BFloat16::ZERO);
    assert!(BFloat16::from(2.0) > BFloat16::ONE);
    assert!(BFloat16::ONE <= BFloat16::ONE);
    assert!(BFloat16::ONE >= BFloat16::ONE);
    assert_eq!(
        BFloat16::ONE.partial_cmp(&BFloat16::from(2.0)),
        Some(Ordering::Less)
    );
    assert_eq!(
        BFloat16::from(2.0).partial_cmp(&BFloat16::ONE),
        Some(Ordering::Greater)
    );
    assert_eq!(
        BFloat16::ONE.partial_cmp(&BFloat16::ONE),
        Some(Ordering::Equal)
    );
}

#[test]
fn test_ordering_agrees_with_f32_on_every_pair_in_a_window() {
    let values: Vec<BFloat16> = (0xBF80..0xBF80 + 32)
        .chain(0x3F80..0x3F80 + 32)
        .chain([0x0000, 0x8000, 0x0001, 0x8001, 0x7F80, 0xFF80, 0x7FC0])
        .map(BFloat16::from_bits)
        .collect();
    for &a in &values {
        for &b in &values {
            let (x, y) = (a.to_f32(), b.to_f32());
            assert_eq!(a.partial_cmp(&b), x.partial_cmp(&y), "{x} vs {y}");
            assert_eq!(a == b, x == y, "{x} == {y}");
            assert_eq!(a < b, x < y, "{x} < {y}");
            assert_eq!(a <= b, x <= y, "{x} <= {y}");
            assert_eq!(a > b, x > y, "{x} > {y}");
            assert_eq!(a >= b, x >= y, "{x} >= {y}");
        }
    }
}

#[test]
fn test_ordering_is_monotone_in_the_bit_pattern_for_positive_finite_values() {
    // For positive non-NaN patterns the value order is the integer order of the bits: an
    // algebraic invariant of the format that does not depend on f32.
    for bits in 0x0000..0x7F80u16 {
        assert!(
            BFloat16::from_bits(bits) < BFloat16::from_bits(bits + 1),
            "{bits:#06x}"
        );
    }
}
