/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::ScalarEval;

#[test]
fn test_f32_modulus_squared() {
    let x: f32 = 2.0;
    assert_eq!(x.modulus_squared(), 4.0);

    let x: f32 = -3.0;
    assert_eq!(x.modulus_squared(), 9.0);

    let x: f32 = 0.0;
    assert_eq!(x.modulus_squared(), 0.0);

    let x: f32 = 0.5;
    assert_eq!(x.modulus_squared(), 0.25);
}

#[test]
fn test_f32_scale_by_real() {
    let x: f32 = 2.0;
    let s: f32 = 3.0;
    assert_eq!(x.scale_by_real(s), 6.0);

    let x: f32 = -4.0;
    let s: f32 = 2.0;
    assert_eq!(x.scale_by_real(s), -8.0);

    let x: f32 = 5.0;
    let s: f32 = 0.0;
    assert_eq!(x.scale_by_real(s), 0.0);

    let x: f32 = 10.0;
    let s: f32 = 0.5;
    assert_eq!(x.scale_by_real(s), 5.0);
}
