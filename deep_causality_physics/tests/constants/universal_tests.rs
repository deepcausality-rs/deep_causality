/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_physics::{G, PLANCK_CONSTANT, SPEED_OF_LIGHT, VACUUM_ELECTRIC_PERMITTIVITY};

#[test]
fn test_universal_constants_approx() {
    // Sanity checks against known rough values
    assert!((SPEED_OF_LIGHT - 299_792_458.0).abs() < 1e-9);
    assert!(PLANCK_CONSTANT > 0.0);
    assert!(VACUUM_ELECTRIC_PERMITTIVITY > 0.0);
    assert!((G - 9.80665).abs() < 1e-5);
}
