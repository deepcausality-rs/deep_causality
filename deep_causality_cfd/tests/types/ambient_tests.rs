/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for `Ambient` — the per-step environment a marcher reads (viscosity, freestream speed,
//! optional body force) and that coupling stages write into between steps.

use deep_causality_cfd::Ambient;

#[test]
fn test_new_and_getters() {
    let a = Ambient::<f64>::new(0.01, 2.5, None);
    assert_eq!(*a.nu(), 0.01);
    assert_eq!(*a.freestream(), 2.5);
    assert!(a.body_force().is_none());
}

#[test]
fn test_setters_drive_state_between_steps() {
    let mut a = Ambient::<f64>::new(0.01, 2.5, None);

    a.set_nu(0.02);
    assert_eq!(*a.nu(), 0.02);

    a.set_freestream(3.0);
    assert_eq!(*a.freestream(), 3.0);

    // Clearing an already-empty body force is a no-op that still drives the field.
    a.set_body_force(None);
    assert!(a.body_force().is_none());
}

#[test]
fn test_clone_and_debug() {
    let a = Ambient::<f64>::new(0.05, 1.0, None);
    let cloned = a.clone();
    assert_eq!(*cloned.nu(), 0.05);
    assert_eq!(*cloned.freestream(), 1.0);
    assert!(format!("{a:?}").contains("Ambient"));
}
