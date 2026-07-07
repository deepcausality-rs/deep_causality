/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num_complex::Complex;
use deep_causality_physics::OrderParameter;

#[test]
fn test_order_parameter() {
    let op = OrderParameter::new(Complex::new(1.0, 1.0));
    assert_eq!(op.value(), Complex::new(1.0, 1.0));
    assert_eq!(op.magnitude_squared(), 2.0);
}
