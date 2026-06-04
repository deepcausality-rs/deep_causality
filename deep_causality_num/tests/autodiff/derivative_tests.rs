/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Real, derivative, second_derivative, value_and_derivative};

const TOL: f64 = 1e-9;

#[test]
fn test_derivative_polynomial() {
    // f(x) = x³ + 2x → f'(3) = 3·3² + 2 = 29
    assert_eq!(derivative(|x| x * x * x + x + x, 3.0_f64), 29.0);
}

#[test]
fn test_derivative_exp_chain_rule() {
    // f(x) = exp(sin x) → f'(x) = cos(x)·exp(sin x)
    let x0 = 0.7_f64;
    let got = derivative(|x| x.sin().exp(), x0);
    let want = x0.cos() * x0.sin().exp();
    assert!((got - want).abs() < TOL);
}

#[test]
fn test_derivative_product_rule() {
    // f(x) = x²·sin(x) → f'(x) = 2x·sin(x) + x²·cos(x)
    let x0 = 1.3_f64;
    let got = derivative(|x| x * x * x.sin(), x0);
    let want = 2.0 * x0 * x0.sin() + x0 * x0 * x0.cos();
    assert!((got - want).abs() < TOL);
}

#[test]
fn test_derivative_quotient_rule() {
    // f(x) = sin(x) / x → f'(x) = (x·cos(x) − sin(x)) / x²
    let x0 = 2.0_f64;
    let got = derivative(|x| x.sin() / x, x0);
    let want = (x0 * x0.cos() - x0.sin()) / (x0 * x0);
    assert!((got - want).abs() < TOL);
}

#[test]
fn test_derivative_ln() {
    // f(x) = ln(x) → f'(x) = 1/x
    let x0 = 5.0_f64;
    let got = derivative(|x| x.ln(), x0);
    assert!((got - 1.0 / x0).abs() < TOL);
}

#[test]
fn test_value_and_derivative_single_pass() {
    let (v, d) = value_and_derivative(|x| x * x, 4.0_f64);
    assert_eq!(v, 16.0); // f(4)
    assert_eq!(d, 8.0); // f'(4) = 2·4
}

#[test]
fn test_second_derivative_quartic() {
    // f(x) = x⁴ → f''(2) = 12·2² = 48
    assert_eq!(second_derivative(|x| x * x * x * x, 2.0_f64), 48.0);
}

#[test]
fn test_second_derivative_sin() {
    // f(x) = sin(x) → f''(x) = −sin(x)
    let x0 = 0.9_f64;
    let got = second_derivative(|x| x.sin(), x0);
    assert!((got - (-x0.sin())).abs() < TOL);
}

#[test]
fn test_derivative_is_precision_generic_f32() {
    // The surface is generic over the scalar: f32 works exactly as f64.
    assert_eq!(derivative(|x| x * x, 3.0_f32), 6.0_f32);
    let (v, d) = value_and_derivative(|x| x * x * x, 2.0_f32);
    assert_eq!(v, 8.0_f32);
    assert_eq!(d, 12.0_f32); // 3·2²
}
