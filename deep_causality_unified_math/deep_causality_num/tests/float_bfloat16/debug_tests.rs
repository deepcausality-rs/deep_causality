/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `Debug` on `BFloat16`, which renders the value the way `f32`'s `Debug` does.

use deep_causality_num::BFloat16;

#[test]
fn test_debug_renders_the_value() {
    assert_eq!(format!("{:?}", BFloat16::from(1.5)), "1.5");
    assert_eq!(format!("{:?}", BFloat16::ONE), "1.0");
    assert_eq!(format!("{:?}", BFloat16::from(-42.0)), "-42.0");
    // 0x3DCD = 0.10009765625. The f32 spacing near 0.1 is 2^-27 = 7.45e-9, so a decimal reads
    // back to this value only if it lies within 3.73e-9 of it. The eight-digit 0.10009766 is
    // 3.75e-9 away and reads back to the next f32; nine digits are the shortest that work.
    assert_eq!(format!("{:?}", BFloat16::from(0.1)), "0.100097656");
}

#[test]
fn test_debug_special_values() {
    assert_eq!(format!("{:?}", BFloat16::NAN), "NaN");
    assert_eq!(format!("{:?}", BFloat16::INFINITY), "inf");
    assert_eq!(format!("{:?}", BFloat16::NEG_INFINITY), "-inf");
    assert_eq!(format!("{:?}", BFloat16::ZERO), "0.0");
    assert_eq!(format!("{:?}", BFloat16::NEG_ZERO), "-0.0");
}

#[test]
fn test_debug_honours_precision() {
    let third = BFloat16::ONE / BFloat16::from(3.0);
    assert_eq!(format!("{third:.2?}"), "0.33");
}
