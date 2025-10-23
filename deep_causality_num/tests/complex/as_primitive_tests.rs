/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{AsPrimitive, Complex};

#[test]
fn test_as_primitive_f64() {
    let c = Complex::new(1.5, 2.5);
    let val: f64 = c.as_();
    assert_eq!(val, 1.5);
}
