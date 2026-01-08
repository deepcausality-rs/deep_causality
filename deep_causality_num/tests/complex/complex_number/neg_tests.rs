/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_num::Complex;

#[test]
fn test_complex_neg() {
    let c = Complex::new(1.0, -2.0);
    let neg_c = -c;
    assert_eq!(neg_c.re(), -1.0);
    assert_eq!(neg_c.im(), 2.0);
}
