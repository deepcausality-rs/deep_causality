/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `Display`, `LowerExp` and `UpperExp` on `BFloat16`.
//!
//! The value is rendered through `f32`'s formatting with the caller's formatter, so every flag
//! the caller sets applies. half-rs #112 reports `{:.1e}` ignoring the precision; the exact case
//! from that report is pinned here.

use deep_causality_num::BFloat16;

#[test]
fn test_display_exact_values() {
    assert_eq!(format!("{}", BFloat16::from(1.5)), "1.5");
    assert_eq!(format!("{}", BFloat16::ONE), "1");
    assert_eq!(format!("{}", BFloat16::from(-2.25)), "-2.25");
    assert_eq!(format!("{}", BFloat16::from(1000.0)), "1000");
}

#[test]
fn test_display_special_values() {
    assert_eq!(format!("{}", BFloat16::NAN), "NaN");
    assert_eq!(format!("{}", BFloat16::INFINITY), "inf");
    assert_eq!(format!("{}", BFloat16::NEG_INFINITY), "-inf");
    assert_eq!(format!("{}", BFloat16::ZERO), "0");
    assert_eq!(format!("{}", BFloat16::NEG_ZERO), "-0");
}

#[test]
fn test_display_honours_precision_width_fill_and_sign() {
    // 1/3 in bf16 is 0.333984375.
    let third = BFloat16::ONE / BFloat16::from(3.0);
    assert_eq!(format!("{third:.3}"), "0.334");
    assert_eq!(format!("{third:.6}"), "0.333984");
    assert_eq!(format!("{:>6}", BFloat16::from(1.5)), "   1.5");
    assert_eq!(format!("{:<6}|", BFloat16::from(1.5)), "1.5   |");
    assert_eq!(format!("{:*^7}", BFloat16::from(1.5)), "**1.5**");
    assert_eq!(format!("{:+}", BFloat16::from(1.5)), "+1.5");
    assert_eq!(format!("{:08.3}", BFloat16::from(-1.5)), "-001.500");
}

#[test]
fn test_display_shows_the_exact_value_the_type_holds() {
    // 0.1 is not representable; the type holds 0x3DCD = 0.10009765625 and prints that value.
    let x = BFloat16::from(0.1);
    assert_eq!(format!("{x:.11}"), "0.10009765625");
    // 0x3DCD = 205/2048; the quotient is exact in f32.
    assert_eq!(format!("{x}"), format!("{}", 205.0_f32 / 2048.0));
}

#[test]
fn test_lower_and_upper_exp() {
    assert_eq!(format!("{:e}", BFloat16::from(1024.0)), "1.024e3");
    assert_eq!(format!("{:E}", BFloat16::from(1024.0)), "1.024E3");
    assert_eq!(format!("{:e}", BFloat16::from(0.5)), "5e-1");
    assert_eq!(format!("{:.2e}", BFloat16::from(1024.0)), "1.02e3");
    assert_eq!(format!("{:e}", BFloat16::NAN), "NaN");
    assert_eq!(format!("{:E}", BFloat16::NEG_INFINITY), "-inf");
}

#[test]
fn test_lower_exp_honours_precision_like_f32() {
    // half-rs #112: `{:.1e}` of pi printed 3.140625e0 where f32 prints 3.1e0.
    let pi = BFloat16::PI;
    assert_eq!(format!("{pi:.1e}"), "3.1e0");
    assert_eq!(format!("{pi:.1e}"), format!("{:.1e}", pi.to_f32()));
    assert_eq!(format!("{pi:.3E}"), "3.141E0");
}
