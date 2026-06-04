/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Dual;

#[test]
fn test_display() {
    let d = Dual::new(1.5_f64, 2.5);
    assert_eq!(format!("{}", d), "1.5 + 2.5ε");
}

#[test]
fn test_display_constant_and_variable() {
    assert_eq!(format!("{}", Dual::constant(3.0_f64)), "3 + 0ε");
    assert_eq!(format!("{}", Dual::variable(3.0_f64)), "3 + 1ε");
}
