/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Octonion;

#[test]
fn test_debug_octonion_f32() {
    let o = Octonion::new(
        1.23f32, 2.34f32, 3.45f32, 4.56f32, 5.67f32, 6.78f32, 7.89f32, 8.90f32,
    );
    assert_eq!(
        format!("{:?}", o),
        "Octonion { s: 1.23, e1: 2.34, e2: 3.45, e3: 4.56, e4: 5.67, e5: 6.78, e6: 7.89, e7: 8.9 }"
    );
}

#[test]
fn test_debug_octonion_f64() {
    let o = Octonion::new(
        1.23456789f64,
        -2.34567890f64,
        3.45678901f64,
        -4.56789012f64,
        5.67890123f64,
        -6.78901234f64,
        7.89012345f64,
        -8.90123456f64,
    );
    assert_eq!(
        format!("{:?}", o),
        "Octonion { s: 1.23456789, e1: -2.3456789, e2: 3.45678901, e3: -4.56789012, e4: 5.67890123, e5: -6.78901234, e6: 7.89012345, e7: -8.90123456 }"
    );
}
