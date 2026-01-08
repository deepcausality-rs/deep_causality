/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::Octonion;

#[test]
fn test_octonion_neg() {
    let o = Octonion::new(1.0, -2.0, 3.0, -4.0, 5.0, -6.0, 7.0, -8.0);
    let neg_o = -o;
    assert_eq!(neg_o.s, -1.0);
    assert_eq!(neg_o.e1, 2.0);
    assert_eq!(neg_o.e2, -3.0);
    assert_eq!(neg_o.e3, 4.0);
    assert_eq!(neg_o.e4, -5.0);
    assert_eq!(neg_o.e5, 6.0);
    assert_eq!(neg_o.e6, -7.0);
    assert_eq!(neg_o.e7, 8.0);
}
