/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Complex;

#[test]
fn test_debug_complex_f32() {
    let c = Complex::new(1.23f32, 4.56f32);
    assert_eq!(format!("{:?}", c), "Complex { re: 1.23, im: 4.56 }");
}

#[test]
fn test_debug_complex_f64() {
    let c = Complex::new(1.23456789f64, -9.87654321f64);
    assert_eq!(
        format!("{:?}", c),
        "Complex { re: 1.23456789, im: -9.87654321 }"
    );
}
