/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Dual;

#[test]
fn test_add() {
    let c = Dual::new(1.0_f64, 2.0) + Dual::new(3.0, 4.0);
    assert_eq!(c.value(), 4.0);
    assert_eq!(c.derivative(), 6.0);
}

#[test]
fn test_sub() {
    let c = Dual::new(1.0_f64, 2.0) - Dual::new(3.0, 4.0);
    assert_eq!(c.value(), -2.0);
    assert_eq!(c.derivative(), -2.0);
}

#[test]
fn test_mul_is_the_product_rule() {
    // (a+bε)(c+dε) = ac + (ad+bc)ε
    let c = Dual::new(2.0_f64, 3.0) * Dual::new(5.0, 7.0);
    assert_eq!(c.value(), 10.0);
    assert_eq!(c.derivative(), 2.0 * 7.0 + 3.0 * 5.0); // 29
}

#[test]
fn test_neg() {
    let n = -Dual::new(2.0_f64, 3.0);
    assert_eq!(n.value(), -2.0);
    assert_eq!(n.derivative(), -3.0);
}

#[test]
fn test_div_is_the_quotient_rule() {
    // (a+bε)/(c+dε) = a/c + (bc − ad)/c² ε
    let q = Dual::new(6.0_f64, 2.0) / Dual::new(3.0, 1.0);
    assert_eq!(q.value(), 2.0); // 6/3
    // (2·3 − 6·1) / 9 = 0
    assert_eq!(q.derivative(), 0.0);
}

#[test]
fn test_assign_ops() {
    let mut a = Dual::new(1.0_f64, 2.0);
    a += Dual::new(3.0, 4.0);
    assert_eq!(a, Dual::new(4.0, 6.0));
    a -= Dual::new(1.0, 1.0);
    assert_eq!(a, Dual::new(3.0, 5.0));

    let mut b = Dual::new(2.0_f64, 3.0);
    b *= Dual::new(5.0, 7.0);
    assert_eq!(b.value(), 10.0);
    assert_eq!(b.derivative(), 29.0);
}

#[test]
fn test_scalar_multiplication() {
    let s = Dual::new(2.0_f64, 3.0) * 4.0;
    assert_eq!(s.value(), 8.0);
    assert_eq!(s.derivative(), 12.0);

    let mut m = Dual::new(2.0_f64, 3.0);
    m *= 4.0;
    assert_eq!(m, Dual::new(8.0, 12.0));
}

#[test]
fn test_forward_mode_ad_polynomial() {
    // f(x) = x³ + 2x at x = 3  →  value 33, derivative 3x² + 2 = 29
    let x = Dual::variable(3.0_f64);
    let y = x * x * x + x + x;
    assert_eq!(y.value(), 33.0);
    assert_eq!(y.derivative(), 29.0);
}

#[test]
fn test_sum_and_product() {
    let v = [
        Dual::new(1.0_f64, 1.0),
        Dual::new(2.0, 1.0),
        Dual::new(3.0, 1.0),
    ];

    let s: Dual<f64> = v.iter().copied().sum();
    assert_eq!(s.value(), 6.0);
    assert_eq!(s.derivative(), 3.0);

    let p: Dual<f64> = v.iter().copied().product();
    assert_eq!(p.value(), 6.0); // 1·2·3
    // d(xyz) = yz + xz + xy = 6 + 3 + 2 = 11
    assert_eq!(p.derivative(), 11.0);
}
