/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::ParticleData;

// =============================================================================
// ParticleData::new constructor + accessor coverage
// =============================================================================

#[test]
fn test_particle_data_new_and_accessors() {
    // Exercises the const constructor body (all field assignments) plus every
    // accessor, including spin() which is otherwise unexercised.
    let p = ParticleData::new(2212, 0.938272081, 1.0, 0.5, "p");

    assert_eq!(p.pdg_id(), 2212);
    assert!((p.mass() - 0.938272081).abs() < 1e-12);
    assert!((p.charge() - 1.0).abs() < 1e-12);
    assert!((p.spin() - 0.5).abs() < 1e-12);
    assert_eq!(p.name(), "p");
}

#[test]
fn test_particle_data_new_const_context() {
    // Construct in a const context to confirm the `const fn` path compiles and
    // all fields are wired correctly for a spin-1 vector meson.
    const RHO: ParticleData = ParticleData::new(113, 0.77526, 0.0, 1.0, "rho0");

    assert_eq!(RHO.pdg_id(), 113);
    assert!((RHO.mass() - 0.77526).abs() < 1e-12);
    assert!((RHO.charge()).abs() < 1e-12);
    assert!((RHO.spin() - 1.0).abs() < 1e-12);
    assert_eq!(RHO.name(), "rho0");
}

#[test]
fn test_particle_data_spin_half_integer() {
    // Delta baryon: spin 3/2 — guards spin() returning a non-trivial value.
    let delta = ParticleData::new(2224, 1.232, 2.0, 1.5, "Delta++");
    assert!((delta.spin() - 1.5).abs() < 1e-12);
    assert!((delta.charge() - 2.0).abs() < 1e-12);
}
