/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_physics::{ATOMIC_MASS_CONSTANT, ELECTRON_MASS, PROTON_MASS};

#[test]
#[allow(clippy::assertions_on_constants)]
fn test_atomic_constants_sanity() {
    // Electron Mass ~ 9.109e-31 kg
    assert!((ELECTRON_MASS - 9.109_383_7e-31).abs() < 1e-38);
    // Proton Mass ~ 1.672e-27 kg
    assert!(PROTON_MASS > ELECTRON_MASS);
    assert!((PROTON_MASS - 1.672_621_9e-27).abs() < 1e-34);
    // Atomic Mass Constant ~ 1.660e-27 kg
    assert!((ATOMIC_MASS_CONSTANT - 1.660_539_0e-27).abs() < 1e-34);
}
