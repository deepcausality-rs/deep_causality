/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AmountOfSubstance, HalfLife, Mass, Time, binding_energy, radioactive_decay,
};

// =============================================================================
// radioactive_decay Wrapper Tests
// =============================================================================

#[test]
fn test_radioactive_decay_wrapper_success() {
    let n0 = AmountOfSubstance::<f64>::new(1000.0).unwrap();
    let half_life = HalfLife::<f64>::new(100.0).unwrap();
    let time = Time::new(50.0).unwrap();

    let effect = radioactive_decay(&n0, &half_life, &time);
    assert!(effect.is_ok());

    let n = effect.value_cloned().unwrap();
    assert!(n.value() > 0.0);
    assert!(n.value() < 1000.0);
}

// =============================================================================
// binding_energy Wrapper Tests
// =============================================================================

#[test]
fn test_radioactive_decay_wrapper_error() {
    // new_unchecked(0.0) bypasses the HalfLife guard so the kernel returns a
    // Singularity error, exercising the wrapper's `Err => from_error` branch.
    let n0 = AmountOfSubstance::<f64>::new(1000.0).unwrap();
    let half_life = HalfLife::<f64>::new_unchecked(0.0);
    let time = Time::new(50.0).unwrap();

    let effect = radioactive_decay(&n0, &half_life, &time);
    assert!(
        effect.is_err(),
        "Zero half-life must propagate as an error effect"
    );
}

#[test]
fn test_binding_energy_wrapper_success() {
    let mass_defect = Mass::new(1e-27).unwrap();

    let effect = binding_energy(&mass_defect);
    assert!(effect.is_ok());

    let energy = effect.value_cloned().unwrap();
    assert!(energy.value() > 0.0);
}

// NOTE on nuclear/wrappers.rs:34 — the `Err(e)` arm of `binding_energy`.
// `binding_energy_kernel` computes `Energy::new(mass_defect · c²)`. `Energy::new`
// is infallible (energy may be any finite or non-finite value; it performs no
// validation and unconditionally returns `Ok`), and `R::from_f64(SPEED_OF_LIGHT)`
// is infallible for f64. The kernel therefore always returns `Ok`, so the
// wrapper's error arm is unreachable.
