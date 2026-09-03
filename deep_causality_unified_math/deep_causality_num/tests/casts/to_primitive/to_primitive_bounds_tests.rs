/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The first value past each wide integer target is refused, not saturated.

use deep_causality_num::ToPrimitive;

#[test]
fn test_two_to_the_64_is_not_a_u64() {
    let beyond = 18_446_744_073_709_551_616.0f64;
    assert_eq!(beyond.to_u64(), None);
    assert_eq!((beyond as f32).to_u64(), None);
    assert_eq!(
        18_446_744_073_709_549_568.0f64.to_u64(),
        Some(18_446_744_073_709_549_568)
    );
}

#[test]
fn test_two_to_the_63_is_not_an_i64() {
    let beyond = 9_223_372_036_854_775_808.0f64;
    assert_eq!(beyond.to_i64(), None);
    assert_eq!((-beyond).to_i64(), Some(i64::MIN));
    assert_eq!((beyond as f32).to_i64(), None);
}

#[test]
fn test_the_128_bit_edges() {
    assert_eq!(3.402_823_669_209_385e38f64.to_u128(), None);
    assert_eq!(1.701_411_834_604_692_3e38f64.to_i128(), None);
    assert_eq!((-1.701_411_834_604_692_3e38f64).to_i128(), Some(i128::MIN));
    assert!(1.7e38f64.to_u128().is_some());
}

#[test]
fn test_the_narrow_targets_keep_their_exact_maxima() {
    assert_eq!(255.0f64.to_u8(), Some(255));
    assert_eq!(256.0f64.to_u8(), None);
    assert_eq!(4_294_967_295.0f64.to_u32(), Some(u32::MAX));
    assert_eq!(4_294_967_296.0f64.to_u32(), None);
}
