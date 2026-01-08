/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Complex, One, Zero};

#[test]
fn test_complex_zero() {
    let c = Complex::<f64>::zero();
    assert_eq!(c.re(), 0.0);
    assert_eq!(c.im(), 0.0);
    assert!(c.is_zero());
}

#[test]
fn test_complex_one() {
    let c = Complex::<f64>::one();
    assert_eq!(c.re(), 1.0);
    assert_eq!(c.im(), 0.0);
    assert!(c.is_one());
}
