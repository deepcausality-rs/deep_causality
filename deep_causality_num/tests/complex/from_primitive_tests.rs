/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Complex, ComplexNumber, FromPrimitive};

#[test]
fn test_from_primitive_f64() {
    let c = Complex::<f64>::from_f64(1.5).unwrap();
    assert_eq!(c.re(), 1.5);
    assert_eq!(c.im(), 0.0);
}
