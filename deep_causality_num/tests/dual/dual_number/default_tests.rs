/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Dual, Zero};

#[test]
fn test_default_is_zero_dual() {
    let d: Dual<f64> = Dual::default();
    // `0 + 0·ε`: value and derivative both zero.
    assert_eq!(d.value(), 0.0);
    assert_eq!(d.derivative(), 0.0);
}

#[test]
fn test_default_agrees_with_zero() {
    // `Default` and `Zero` must coincide (the additive identity).
    let d: Dual<f64> = Dual::default();
    assert_eq!(d, Dual::<f64>::zero());
    assert!(d.is_zero());
}

#[test]
fn test_default_equals_explicit_constant() {
    let d: Dual<f64> = Dual::default();
    assert_eq!(d, Dual::new(0.0, 0.0));
    assert_eq!(d, Dual::constant(0.0));
}

#[test]
fn test_default_nests() {
    // `Dual<Dual<f64>>` (second-order AD carrier) also defaults to a full zero.
    let d: Dual<Dual<f64>> = Dual::default();
    assert_eq!(d, Dual::<Dual<f64>>::zero());
    assert!(d.is_zero());
    assert_eq!(d.value().value(), 0.0);
    assert_eq!(d.value().derivative(), 0.0);
    assert_eq!(d.derivative().value(), 0.0);
    assert_eq!(d.derivative().derivative(), 0.0);
}
