/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::{Complex, ToPrimitive};

// Conversion trait tests
#[test]
fn test_to_primitive_f64() {
    let c = Complex::new(1.5, 2.5);
    assert_eq!(c.to_f64(), Some(1.5));
}
