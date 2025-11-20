/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_num::Octonion;

#[test]
fn test_display_octonion_positive_all() {
    let o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    assert_eq!(
        format!("{}", o),
        "1 + 2e₁ + 3e₂ + 4e₃ + 5e₄ + 6e₅ + 7e₆ + 8e₇"
    );
}

#[test]
fn test_display_octonion_negative_some() {
    let o = Octonion::new(1.0, -2.0, 3.0, -4.0, 5.0, -6.0, 7.0, -8.0);
    assert_eq!(
        format!("{}", o),
        "1 - 2e₁ + 3e₂ - 4e₃ + 5e₄ - 6e₅ + 7e₆ - 8e₇"
    );
}

#[test]
fn test_display_octonion_zero_some() {
    let o = Octonion::new(0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0, 0.0);
    assert_eq!(format!("{}", o), "1e₂ - 1e₆");
}

#[test]
fn test_display_octonion_zero_all() {
    let o = Octonion::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(format!("{}", o), "0");
}

#[test]
fn test_display_octonion_scalar_only() {
    let o = Octonion::new(1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    assert_eq!(format!("{}", o), "1");
}
