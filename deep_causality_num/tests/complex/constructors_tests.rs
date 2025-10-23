/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::{Complex, ComplexNumber};

#[test]
fn test_complex_new() {
    let c = Complex::new(1.0, 2.0);
    assert_eq!(c.re(), 1.0);
    assert_eq!(c.im(), 2.0);
}

#[test]
fn test_complex_from_real() {
    let c = Complex::from_real(3.0);
    assert_eq!(c.re(), 3.0);
    assert_eq!(c.im(), 0.0);
}
