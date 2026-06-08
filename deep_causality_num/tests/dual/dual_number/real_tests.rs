/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Dual, Real};

const TOL: f64 = 1e-9;

#[test]
fn test_exp_derivative() {
    let y = Dual::variable(1.5_f64).exp();
    assert!((y.value() - 1.5_f64.exp()).abs() < TOL);
    assert!((y.derivative() - 1.5_f64.exp()).abs() < TOL); // d/dx eˣ = eˣ
}

#[test]
fn test_ln_derivative() {
    let y = Dual::variable(2.0_f64).ln();
    assert!((y.value() - 2.0_f64.ln()).abs() < TOL);
    assert!((y.derivative() - 0.5).abs() < TOL); // 1/2
}

#[test]
fn test_sqrt_derivative() {
    let y = Dual::variable(4.0_f64).sqrt();
    assert!((y.value() - 2.0).abs() < TOL);
    assert!((y.derivative() - 0.25).abs() < TOL); // 1/(2·√4)
}

#[test]
fn test_sin_and_cos_derivatives() {
    let s = Dual::variable(0.5_f64).sin();
    assert!((s.derivative() - 0.5_f64.cos()).abs() < TOL);
    let c = Dual::variable(0.5_f64).cos();
    assert!((c.derivative() - (-0.5_f64.sin())).abs() < TOL);
}

#[test]
fn test_tan_derivative() {
    let y = Dual::variable(0.3_f64).tan();
    let sec2 = 1.0 / (0.3_f64.cos() * 0.3_f64.cos());
    assert!((y.derivative() - sec2).abs() < TOL);
}

#[test]
fn test_inverse_trig_and_hyperbolic_derivatives() {
    let a = 0.4_f64;
    let d = (1.0 - a * a).sqrt();
    assert!((Dual::variable(a).asin().derivative() - 1.0 / d).abs() < TOL);
    assert!((Dual::variable(a).acos().derivative() - (-1.0 / d)).abs() < TOL);
    assert!((Dual::variable(a).atan().derivative() - 1.0 / (1.0 + a * a)).abs() < TOL);
    assert!((Dual::variable(a).sinh().derivative() - a.cosh()).abs() < TOL);
    assert!((Dual::variable(a).cosh().derivative() - a.sinh()).abs() < TOL);
    assert!((Dual::variable(a).tanh().derivative() - (1.0 - a.tanh() * a.tanh())).abs() < TOL);
}

#[test]
fn test_log_base_derivative() {
    // d/dx log₂(x) = 1/(x·ln2); d/dx log₁₀(x) = 1/(x·ln10)
    let x = 8.0_f64;
    assert!((Dual::variable(x).log2().derivative() - 1.0 / (x * 2.0_f64.ln())).abs() < TOL);
    assert!((Dual::variable(x).log10().derivative() - 1.0 / (x * 10.0_f64.ln())).abs() < TOL);
    // log with an arbitrary (constant) base
    let y = Dual::variable(x).log(Dual::constant(2.0));
    assert!((y.value() - x.log(2.0)).abs() < TOL);
    assert!((y.derivative() - 1.0 / (x * 2.0_f64.ln())).abs() < TOL);
}

#[test]
fn test_chain_rule_through_sin_times_exp() {
    // f(x) = sin(x)·exp(x); f'(x) = cos(x)·exp(x) + sin(x)·exp(x)
    let x0 = 0.7_f64;
    let y = Dual::variable(x0).sin() * Dual::variable(x0).exp();
    let expected = x0.cos() * x0.exp() + x0.sin() * x0.exp();
    assert!((y.derivative() - expected).abs() < TOL);
}

#[test]
fn test_powf_derivative() {
    // f(x) = x^2.5 at x = 3 → f'(x) = 2.5·x^1.5
    let y = Dual::variable(3.0_f64).powf(Dual::constant(2.5));
    assert!((y.value() - 3.0_f64.powf(2.5)).abs() < 1e-6);
    assert!((y.derivative() - 2.5 * 3.0_f64.powf(1.5)).abs() < 1e-6);
}

#[test]
fn test_abs_derivative_is_sign() {
    assert_eq!(Dual::variable(2.0_f64).abs().derivative(), 1.0);
    let n = Dual::variable(-2.0_f64).abs();
    assert_eq!(n.value(), 2.0);
    assert_eq!(n.derivative(), -1.0);
}

#[test]
fn test_nonsmooth_ops_have_zero_derivative() {
    assert_eq!(Dual::variable(2.7_f64).floor().value(), 2.0);
    assert_eq!(Dual::variable(2.7_f64).floor().derivative(), 0.0);
    assert_eq!(Dual::variable(2.1_f64).ceil().value(), 3.0);
    assert_eq!(Dual::variable(2.1_f64).ceil().derivative(), 0.0);
    assert_eq!(Dual::variable(2.5_f64).round().derivative(), 0.0);
}

#[test]
fn test_constants_and_predicates() {
    assert_eq!(<Dual<f64> as Real>::pi().value(), core::f64::consts::PI);
    assert_eq!(<Dual<f64> as Real>::pi().derivative(), 0.0);
    assert_eq!(<Dual<f64> as Real>::e().value(), core::f64::consts::E);
    assert!(<Dual<f64> as Real>::nan().is_nan());
    assert!(Dual::variable(1.0_f64).is_finite());
    assert!(Dual::new(f64::INFINITY, 0.0).is_infinite());
    assert!(!Dual::new(f64::INFINITY, 0.0).is_finite());
}

#[test]
fn test_clamp() {
    let lo = Dual::constant(0.0_f64);
    let hi = Dual::constant(1.0_f64);
    assert_eq!(Dual::variable(0.5_f64).clamp(lo, hi).value(), 0.5);
    assert_eq!(Dual::variable(0.5_f64).clamp(lo, hi).derivative(), 1.0); // passthrough
    assert_eq!(Dual::variable(2.0_f64).clamp(lo, hi).value(), 1.0); // clamped to hi
    assert_eq!(Dual::variable(2.0_f64).clamp(lo, hi).derivative(), 0.0); // bound's derivative
}

#[test]
fn test_clamp_to_lower_bound() {
    let lo = Dual::constant(0.0_f64);
    let hi = Dual::constant(1.0_f64);
    // A value below the lower bound clamps to `min`, carrying the bound's (zero) derivative.
    let r = Dual::variable(-1.0_f64).clamp(lo, hi);
    assert_eq!(r.value(), 0.0);
    assert_eq!(r.derivative(), 0.0);
}

#[test]
fn test_atan2_derivative() {
    // atan2(y, x) with y the variable and x constant: d/dy = x / (x² + y²).
    let y = 1.0_f64;
    let x = 2.0_f64;
    let r = Dual::variable(y).atan2(Dual::constant(x));
    assert!((r.value() - y.atan2(x)).abs() < TOL);
    assert!((r.derivative() - x / (x * x + y * y)).abs() < TOL);
}

#[test]
fn test_epsilon_is_a_constant() {
    assert_eq!(<Dual<f64> as Real>::epsilon().value(), f64::EPSILON);
    assert_eq!(<Dual<f64> as Real>::epsilon().derivative(), 0.0);
}

#[test]
fn test_dual_is_a_real_scalar_and_nests() {
    fn assert_real<T: Real>() {}
    assert_real::<Dual<f64>>();
    assert_real::<Dual<f32>>();
    assert_real::<Dual<Dual<f64>>>(); // duals nest: Dual<Dual<T>> is also Real
}

#[test]
fn test_nested_duals_give_second_derivative() {
    // f(x) = x⁴ at x = 2:  f'(x) = 4x³ = 32,  f''(x) = 12x² = 48
    let x = Dual::variable(Dual::variable(2.0_f64));
    let y = x * x * x * x;
    assert_eq!(y.derivative().value(), 32.0); // first derivative
    assert_eq!(y.derivative().derivative(), 48.0); // second derivative
}
