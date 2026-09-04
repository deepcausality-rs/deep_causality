/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Real` and `DivisionAlgebra` reached through the trait on `BFloat16`, with values derived by
//! hand from the bf16 grid: in `[2^k, 2^(k+1))` the step is `2^(k-7)`.

use deep_causality_algebra::{DivisionAlgebra, Real};
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

// =============================================================================
// Real: special values and classification
// =============================================================================

#[test]
fn test_real_nan_and_classification() {
    let nan = <BFloat16 as Real>::nan();
    assert!(Real::is_nan(nan));
    assert!(!Real::is_nan(bf(1.0)));
    assert!(Real::is_infinite(BFloat16::INFINITY));
    assert!(!Real::is_infinite(bf(1.0)));
    assert!(Real::is_finite(bf(1.0)));
    assert!(!Real::is_finite(BFloat16::INFINITY));
    assert!(!Real::is_finite(nan));
}

// =============================================================================
// Real: clamp, all branches
// =============================================================================

#[test]
fn test_real_clamp() {
    let (lo, hi) = (BFloat16::ZERO, bf(10.0));
    assert_eq!(Real::clamp(bf(5.0), lo, hi), bf(5.0));
    assert_eq!(Real::clamp(bf(-5.0), lo, hi), lo);
    assert_eq!(Real::clamp(bf(15.0), lo, hi), hi);
    assert_eq!(Real::clamp(lo, lo, hi), lo);
    assert_eq!(Real::clamp(hi, lo, hi), hi);
}

// =============================================================================
// Real: functions
// =============================================================================

#[test]
fn test_real_functions() {
    assert_eq!(Real::abs(bf(-42.0)), bf(42.0));
    assert_eq!(Real::sqrt(bf(4.0)), bf(2.0));
    // 3.75 = 11.11b is exact; its floor, ceil and round are 3, 4 and 4.
    assert_eq!(Real::floor(bf(3.75)), bf(3.0));
    assert_eq!(Real::ceil(bf(3.75)), bf(4.0));
    assert_eq!(Real::round(bf(3.5)), bf(4.0));
    // e = 2.71828 rounds to 2 + 46/64 = 2.71875 = 0x402E; ln of that is 1.00017, which rounds
    // to 1 on a grid with step 2^-7.
    assert_eq!(Real::exp(bf(1.0)).to_bits(), 0x402E);
    assert_eq!(Real::ln(BFloat16::E), bf(1.0));
    assert_eq!(Real::log(bf(100.0), bf(10.0)), bf(2.0));
    assert_eq!(Real::log2(bf(8.0)), bf(3.0));
    assert_eq!(Real::log10(bf(1000.0)), bf(3.0));
    assert_eq!(Real::powf(bf(2.0), bf(3.0)), bf(8.0));
}

#[test]
fn test_real_trigonometry() {
    assert_eq!(Real::sin(BFloat16::ZERO).to_bits(), 0x0000);
    assert_eq!(Real::cos(BFloat16::ZERO), bf(1.0));
    assert_eq!(Real::tan(BFloat16::ZERO).to_bits(), 0x0000);
    assert_eq!(Real::acos(bf(1.0)).to_bits(), 0x0000);
    // atan2(1, 1) = pi/4 = 0.7853982; in [0.5, 1) the step is 2^-8 and (0.7853982 - 0.5) / 2^-8
    // = 73.06, so 0.5 + 73/256 = 0.78515625 = 0x3F49.
    assert_eq!(Real::atan2(bf(1.0), bf(1.0)).to_bits(), 0x3F49);
    assert_eq!(Real::sinh(BFloat16::ZERO).to_bits(), 0x0000);
    assert_eq!(Real::cosh(BFloat16::ZERO), bf(1.0));
    assert_eq!(Real::tanh(BFloat16::ZERO).to_bits(), 0x0000);
}

#[test]
fn test_real_constants() {
    // pi = 3.14159 rounds to 2 + 73/64 = 3.140625 = 0x4049.
    assert_eq!(<BFloat16 as Real>::pi().to_bits(), 0x4049);
    assert_eq!(<BFloat16 as Real>::e().to_bits(), 0x402E);
    // The gap above one is 2^-7.
    assert_eq!(<BFloat16 as Real>::epsilon().to_f32(), 0.0078125);
}

// =============================================================================
// DivisionAlgebra
// =============================================================================

#[test]
fn test_division_algebra() {
    let x = bf(5.0);
    assert_eq!(DivisionAlgebra::conjugate(&x), x);
    assert_eq!(DivisionAlgebra::norm_sqr(&bf(3.0)), bf(9.0));
    assert_eq!(DivisionAlgebra::inverse(&bf(4.0)), bf(0.25));
    let four = bf(4.0);
    assert_eq!(four * four.inverse(), bf(1.0));
    // 1/3 = 0x3EAB = 0.333984375, and 3 * 0.333984375 = 1.001953125, which lies below the
    // midpoint 1 + 2^-8 and rounds back to 1.
    let three = bf(3.0);
    assert_eq!(three.inverse().to_bits(), 0x3EAB);
    assert_eq!(three * three.inverse(), bf(1.0));
}
