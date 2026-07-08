/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_dual::Dual;

#[test]
fn test_from_f64_is_a_constant_dual() {
    // A real literal becomes a constant dual: x + 0·ε (zero derivative).
    let d: Dual<f64> = Dual::from(3.5_f64);
    assert_eq!(d.value(), 3.5);
    assert_eq!(d.derivative(), 0.0);
    assert_eq!(d, Dual::constant(3.5));
}

#[test]
fn test_from_f64_via_into() {
    let d: Dual<f64> = 2.0_f64.into();
    assert_eq!(d, Dual::constant(2.0));
}

#[test]
fn test_from_f64_constant_has_no_derivative_under_autodiff() {
    // A constant built via `From` must not contaminate the ε channel: differentiating
    // f(x) = x + 5 (the 5 arriving through `From<f64>`) gives f'(x) = 1.
    let x = Dual::variable(4.0_f64);
    let c: Dual<f64> = Dual::from(5.0_f64);
    let y = x + c;
    assert_eq!(y.value(), 9.0);
    assert_eq!(y.derivative(), 1.0);
}
