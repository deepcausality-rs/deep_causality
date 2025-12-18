/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    AmountOfSubstance, EnergyDensity, HalfLife, Mass, SPEED_OF_LIGHT, Time, binding_energy_kernel,
    hadronization_kernel, radioactive_decay_kernel,
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
    // Zero half-life is now rejected at construction time
    // This test verifies that HalfLife::new(0.0) correctly returns an error
    let half_life_result = HalfLife::new(0.0);
    assert!(
        half_life_result.is_err(),
        "Zero half-life should be rejected at construction"
    );

    match half_life_result {
        Err(e) => match e.0 {
            deep_causality_physics::PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
                assert!(msg.contains("positive") || msg.contains("zero"));
            }
            _ => panic!("Expected PhysicalInvariantBroken error, got {:?}", e),
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

// =============================================================================
// hadronization_kernel Tests
// =============================================================================

#[test]
fn test_hadronization_kernel_valid() {
    let densities = vec![
        EnergyDensity::new(10.0).unwrap(),
        EnergyDensity::new(5.0).unwrap(),
        EnergyDensity::new(20.0).unwrap(),
    ];
    let threshold = 8.0;
    let dim = 3;

    let result = hadronization_kernel(&densities, threshold, dim);
    assert!(result.is_ok());

    let particles = result.unwrap();
    // 10.0 and 20.0 are > 8.0, so 2 particles expected.
    assert_eq!(particles.len(), 2);

    // Verify properties of generated particles
    let p1 = &particles[0];
    let vec1 = p1.0.data();
    // Component 1 should hold the energy density value
    assert!((vec1[1] - 10.0).abs() < 1e-6);

    let p2 = &particles[1];
    let vec2 = p2.0.data();
    assert!((vec2[1] - 20.0).abs() < 1e-6);
}

#[test]
fn test_hadronization_sub_threshold() {
    let densities = vec![
        EnergyDensity::new(1.0).unwrap(),
        EnergyDensity::new(5.0).unwrap(),
    ];
    let threshold = 10.0;
    let dim = 3;

    let result = hadronization_kernel(&densities, threshold, dim);
    assert!(result.is_ok());

    let particles = result.unwrap();
    assert!(
        particles.is_empty(),
        "Should produce no particles below threshold"
    );
}

#[test]
fn test_hadronization_invalid_dim() {
    let densities = vec![EnergyDensity::new(10.0).unwrap()];
    let threshold = 5.0;
    let dim = 0; // Invalid

    let result = hadronization_kernel(&densities, threshold, dim);
    assert!(result.is_err());

    match result {
        Err(e) => match e.0 {
            deep_causality_physics::PhysicsErrorEnum::DimensionMismatch(_) => (),
            _ => panic!("Expected DimensionMismatch, got {:?}", e),
        },
        Ok(_) => panic!("Should fail with dim=0"),
    }
}

#[test]
fn test_hadronization_negative_threshold() {
    let densities = vec![EnergyDensity::new(10.0).unwrap()];
    let threshold = -1.0; // Invalid
    let dim = 3;

    let result = hadronization_kernel(&densities, threshold, dim);
    assert!(result.is_err());
}
