/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AmountOfSubstance, HalfLife, Mass, SPEED_OF_LIGHT, Time, binding_energy_kernel,
    radioactive_decay_kernel,
};

// =============================================================================
// radioactive_decay_kernel Tests
// =============================================================================

#[test]
fn test_radioactive_decay_kernel_valid() {
    // N(t) = N0 × (1/2)^(t/t_half)
    let n0 = AmountOfSubstance::new(1000.0).unwrap();
    let half_life = HalfLife::new(5730.0).unwrap(); // C-14 half-life in years (approx)
    let time = Time::new(5730.0).unwrap(); // One half-life

    let result = radioactive_decay_kernel(&n0, &half_life, &time);
    assert!(result.is_ok());

    let n_t = result.unwrap();
    // After one half-life, should have exactly half
    assert!(
        (n_t.value() - 500.0).abs() < 1e-10,
        "After one half-life, N should be N0/2. Got {}",
        n_t.value()
    );
}

#[test]
fn test_radioactive_decay_two_half_lives() {
    let n0 = AmountOfSubstance::new(1000.0).unwrap();
    let half_life = HalfLife::new(100.0).unwrap();
    let time = Time::new(200.0).unwrap(); // Two half-lives

    let result = radioactive_decay_kernel(&n0, &half_life, &time);
    assert!(result.is_ok());

    let n_t = result.unwrap();
    // After two half-lives: N = N0 × (1/2)^2 = N0/4
    assert!(
        (n_t.value() - 250.0).abs() < 1e-10,
        "After two half-lives, N should be N0/4. Got {}",
        n_t.value()
    );
}

#[test]
fn test_radioactive_decay_zero_time() {
    let n0 = AmountOfSubstance::new(1000.0).unwrap();
    let half_life = HalfLife::new(100.0).unwrap();
    let time = Time::new(0.0).unwrap();

    let result = radioactive_decay_kernel(&n0, &half_life, &time);
    assert!(result.is_ok());

    let n_t = result.unwrap();
    // At t=0, N = N0
    assert!(
        (n_t.value() - 1000.0).abs() < 1e-10,
        "At t=0, N should equal N0"
    );
}

#[test]
fn test_radioactive_decay_zero_half_life() {
    // Zero half-life means instant decay to 0
    let n0 = AmountOfSubstance::new(1000.0).unwrap();
    let half_life = HalfLife::new(0.0).unwrap();
    let time = Time::new(1.0).unwrap();

    let result = radioactive_decay_kernel(&n0, &half_life, &time);
    assert!(result.is_err());

    match result {
        Err(e) => match e.0 {
            deep_causality_physics::PhysicsErrorEnum::Singularity(_) => assert!(true),
            _ => panic!("Expected Singularity error, got {:?}", e),
        },
        Ok(_) => panic!("Should return error for zero half-life"),
    }
}

/// Physics invariant: Decay is monotonically decreasing with time
#[test]
fn test_radioactive_decay_monotonic_decrease() {
    let n0 = AmountOfSubstance::new(1000.0).unwrap();
    let half_life = HalfLife::new(100.0).unwrap();

    let t1 = Time::new(50.0).unwrap();
    let t2 = Time::new(100.0).unwrap();
    let t3 = Time::new(200.0).unwrap();

    let n1 = radioactive_decay_kernel(&n0, &half_life, &t1).unwrap();
    let n2 = radioactive_decay_kernel(&n0, &half_life, &t2).unwrap();
    let n3 = radioactive_decay_kernel(&n0, &half_life, &t3).unwrap();

    assert!(n1.value() > n2.value(), "N(t1) > N(t2)");
    assert!(n2.value() > n3.value(), "N(t2) > N(t3)");
}

// =============================================================================
// binding_energy_kernel Tests
// =============================================================================

#[test]
fn test_binding_energy_kernel_valid() {
    // E = mc²
    let mass_defect = Mass::new(1e-27).unwrap(); // Small mass

    let result = binding_energy_kernel(&mass_defect);
    assert!(result.is_ok());

    let energy = result.unwrap();
    // E = m × c²
    let expected = mass_defect.value() * SPEED_OF_LIGHT * SPEED_OF_LIGHT;
    assert!(
        (energy.value() - expected).abs() < 1.0,
        "Expected {}, got {}",
        expected,
        energy.value()
    );
}

#[test]
fn test_binding_energy_kernel_zero_mass() {
    let mass_defect = Mass::new(0.0).unwrap();

    let result = binding_energy_kernel(&mass_defect);
    assert!(result.is_ok());

    let energy = result.unwrap();
    assert!(
        energy.value().abs() < 1e-10,
        "Zero mass defect → zero binding energy"
    );
}

/// Physics invariant: E = mc² with known value
#[test]
fn test_binding_energy_mc2_formula() {
    // 1 amu ≈ 1.66054e-27 kg
    let one_amu = Mass::new(1.66054e-27).unwrap();

    let result = binding_energy_kernel(&one_amu);
    assert!(result.is_ok());

    let energy = result.unwrap();
    // Expected: ~931 MeV ≈ 1.49e-10 J
    let c = SPEED_OF_LIGHT;
    let expected = 1.66054e-27 * c * c;

    assert!(
        (energy.value() - expected).abs() / expected < 1e-6,
        "E=mc² calculation mismatch"
    );
}
