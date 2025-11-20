/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::Octonion;

#[test]
fn test_octonion_new() {
    let o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    assert_eq!(o.s, 1.0);
    assert_eq!(o.e1, 2.0);
    assert_eq!(o.e2, 3.0);
    assert_eq!(o.e3, 4.0);
    assert_eq!(o.e4, 5.0);
    assert_eq!(o.e5, 6.0);
    assert_eq!(o.e6, 7.0);
    assert_eq!(o.e7, 8.0);
}

#[test]
fn test_octonion_identity() {
    let o = Octonion::<f64>::identity();
    assert_eq!(o.s, 1.0);
    assert_eq!(o.e1, 0.0);
    assert_eq!(o.e2, 0.0);
    assert_eq!(o.e3, 0.0);
    assert_eq!(o.e4, 0.0);
    assert_eq!(o.e5, 0.0);
    assert_eq!(o.e6, 0.0);
    assert_eq!(o.e7, 0.0);
}
