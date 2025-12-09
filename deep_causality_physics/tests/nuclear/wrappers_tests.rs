/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    binding_energy, radioactive_decay,
    AmountOfSubstance, HalfLife, Mass, Time,
};

// =============================================================================
// radioactive_decay Wrapper Tests
// =============================================================================

#[test]
fn test_radioactive_decay_wrapper_success() {
    let n0 = AmountOfSubstance::new(1000.0).unwrap();
    let half_life = HalfLife::new(100.0).unwrap();
    let time = Time::new(50.0).unwrap();

    let effect = radioactive_decay(&n0, &half_life, &time);
    assert!(effect.is_ok());

    let n = effect.value().clone().into_value().unwrap();
    assert!(n.value() > 0.0);
    assert!(n.value() < 1000.0);
}

// =============================================================================
// binding_energy Wrapper Tests
// =============================================================================

#[test]
fn test_binding_energy_wrapper_success() {
    let mass_defect = Mass::new(1e-27).unwrap();

    let effect = binding_energy(&mass_defect);
    assert!(effect.is_ok());

    let energy = effect.value().clone().into_value().unwrap();
    assert!(energy.value() > 0.0);
}
