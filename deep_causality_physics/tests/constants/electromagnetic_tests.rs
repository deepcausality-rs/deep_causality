/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_physics::{ELEMENTARY_CHARGE, FINE_STRUCTURE_CONSTANT};

#[test]
fn test_electromagnetic_constants_sanity() {
    // Elementary Charge ~ 1.602e-19 C
    assert!((ELEMENTARY_CHARGE - 1.602_176_63e-19).abs() < 1e-26);
    // Fine Structure Constant ~ 1/137
    let alpha_inv = 1.0 / FINE_STRUCTURE_CONSTANT;
    assert!((alpha_inv - 137.035_999).abs() < 1e-5);
}
