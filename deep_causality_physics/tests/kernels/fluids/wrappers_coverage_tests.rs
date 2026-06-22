/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Coverage tests for reachable error arms in `kernels::fluids::wrappers`.

use deep_causality_physics::{
    Density, Length, Pressure, Speed, dynamic_pressure, hydrostatic_pressure,
};

// =============================================================================
// hydrostatic_pressure error arm (wrappers.rs:30)
// =============================================================================

#[test]
fn test_hydrostatic_pressure_wrapper_error_path() {
    // ρ·g·h overflows to +∞ (both ρ and h finite but enormous), so the total
    // pressure is non-finite and `Pressure::new` rejects it; the wrapper
    // forwards the error effect (wrappers.rs:30).
    let p0 = Pressure::<f64>::new(0.0).unwrap();
    let density = Density::<f64>::new(1.0e200).unwrap();
    let depth = Length::<f64>::new(1.0e200).unwrap();

    let effect = hydrostatic_pressure(&p0, &density, &depth);
    assert!(!effect.is_ok());
}

// =============================================================================
// dynamic_pressure error arm (wrappers.rs:939)
// =============================================================================

#[test]
fn test_dynamic_pressure_wrapper_error_path() {
    // q = 0.5·ρ·u² overflows to +∞ for a huge (but finite) speed, so
    // `Pressure::new` rejects the non-finite result and the wrapper forwards
    // the error effect (wrappers.rs:939).
    let rho = Density::<f64>::new(1.0e200).unwrap();
    let u = Speed::<f64>::new(1.0e200).unwrap();

    let effect = dynamic_pressure(&rho, &u);
    assert!(!effect.is_ok());
}

// NOTE on defensively-unreachable error arms in `kernels::fluids::wrappers`.
// Each line below is the `Err(e) => from_error(...)` arm of a wrapper whose
// underlying kernel cannot return `Err` for f64 inputs:
//   * wrappers.rs:63  (strain_rate_tensor), 76 (rotation_rate_tensor),
//     97 (velocity_gradient_invariants), 116 (enstrophy_density),
//     222 (kinetic_energy_density), 585 (turbulent_kinetic_energy),
//     599 (dissipation_rate), 699 (q_criterion), 710 (delta_criterion),
//     721 (lambda2), 732 (swirling_strength), 776 (total_enthalpy).
// Every one of these kernels has a single `Result` failure mode: an
// `R::from_f64(<literal>)` (or, transitively, `velocity_gradient_invariants_
// kernel`, which itself only fails on `R::from_f64(0.5)`). `from_f64` is
// infallible for f64, so for the f64 monomorphisation used throughout the
// physics test-suite these wrapper error arms can never run. They exist purely
// to forward errors for hypothetical lower-precision real fields whose
// `from_f64` could fail.
