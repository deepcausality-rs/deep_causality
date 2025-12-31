/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::{
    Activity, AmountOfSubstance, EnergyDensity, FourMomentum, Hadron, HalfLife, LundParameters,
    PhysicsErrorEnum,
};

// =============================================================================
// AmountOfSubstance Tests
// =============================================================================

#[test]
fn test_amount_of_substance_new_valid() {
    let amount = AmountOfSubstance::new(1.0);
    assert!(amount.is_ok());
    assert!((amount.unwrap().value() - 1.0).abs() < 1e-10);
}

#[test]
fn test_amount_of_substance_new_negative_error() {
    let amount = AmountOfSubstance::new(-1.0);
    assert!(amount.is_err());
    match &amount.unwrap_err().0 {
        PhysicsErrorEnum::PhysicalInvariantBroken(msg) => {
            assert!(msg.contains("AmountOfSubstance") || msg.contains("Negative"));
        }
        _ => panic!("Expected PhysicalInvariantBroken error"),
    }
}

#[test]
fn test_amount_of_substance_new_unchecked() {
    let amount = AmountOfSubstance::new_unchecked(5.0);
    assert!((amount.value() - 5.0).abs() < 1e-10);
}

#[test]
fn test_amount_of_substance_into_f64() {
    let amount = AmountOfSubstance::new(2.5).unwrap();
    let val: f64 = amount.into();
    assert!((val - 2.5).abs() < 1e-10);
}

// =============================================================================
// HalfLife Tests
// =============================================================================

#[test]
fn test_half_life_new_valid() {
    let hl = HalfLife::new(5730.0); // C-14
    assert!(hl.is_ok());
}

#[test]
fn test_half_life_new_zero() {
    // Zero half-life is invalid because it implies infinite decay rate
    let hl = HalfLife::new(0.0);
    assert!(hl.is_err(), "Zero half-life should be rejected");
}

#[test]
fn test_half_life_new_negative_error() {
    let hl = HalfLife::new(-100.0);
    assert!(hl.is_err());
}

#[test]
fn test_half_life_new_unchecked() {
    let hl = HalfLife::new_unchecked(1600.0); // Ra-226
    assert!((hl.value() - 1600.0).abs() < 1e-10);
}

#[test]
fn test_half_life_from_f64() {
    let hl = HalfLife::new(123.0).unwrap();
    let val: f64 = hl.into();
    assert!((val - 123.0).abs() < 1e-10);
}

// =============================================================================
// Activity Tests
// =============================================================================

#[test]
fn test_activity_new_valid() {
    let activity = Activity::new(3.7e10); // 1 Curie in Becquerels
    assert!(activity.is_ok());
}

#[test]
fn test_activity_new_negative_error() {
    let activity = Activity::new(-1.0);
    assert!(activity.is_err());
}

#[test]
fn test_activity_new_unchecked() {
    let activity = Activity::new_unchecked(1e6);
    assert!((activity.value() - 1e6).abs() < 1.0);
}

#[test]
fn test_activity_from_f64() {
    let activity = Activity::new(500.0).unwrap();
    let val: f64 = activity.into();
    assert!((val - 500.0).abs() < 1e-10);
}

// =============================================================================
// EnergyDensity Tests
// =============================================================================

#[test]
fn test_energy_density_new_valid() {
    let ed = EnergyDensity::new(100.0);
    assert!(ed.is_ok());
    assert!((ed.unwrap().value() - 100.0).abs() < 1e-10);
}

#[test]
fn test_energy_density_new_negative_error() {
    let ed = EnergyDensity::new(-50.0);
    assert!(ed.is_err());
}

#[test]
fn test_energy_density_unchecked() {
    let ed = EnergyDensity::new_unchecked(25.0);
    assert!((ed.value() - 25.0).abs() < 1e-10);
}

#[test]
fn test_energy_density_into_f64() {
    let ed = EnergyDensity::new(10.0).unwrap();
    let val: f64 = ed.into();
    assert!((val - 10.0).abs() < 1e-10);
}

// =============================================================================
// FourMomentum Tests
// =============================================================================

#[test]
fn test_four_momentum_creation_and_accessors() {
    let p = FourMomentum::new(10.0, 1.0, 2.0, 3.0);
    assert_eq!(p.e(), 10.0);
    assert_eq!(p.px(), 1.0);
    assert_eq!(p.py(), 2.0);
    assert_eq!(p.pz(), 3.0);
}

#[test]
fn test_four_momentum_at_rest() {
    let p = FourMomentum::at_rest(5.0);
    assert_eq!(p.e(), 5.0);
    assert_eq!(p.invariant_mass(), 5.0);
    assert_eq!(p.momentum_magnitude(), 0.0);
}

#[test]
fn test_four_momentum_from_mass_and_momentum() {
    let mass = 3.0; // m=3
    let px = 4.0; // px=4
    let py = 0.0;
    let pz = 0.0;
    // E should be sqrt(3^2 + 4^2) = sqrt(9+16) = 5
    let p = FourMomentum::from_mass_and_momentum(mass, px, py, pz);
    assert!((p.e() - 5.0).abs() < 1e-10);
    assert!((p.invariant_mass() - 3.0).abs() < 1e-10);
}

#[test]
fn test_four_momentum_math() {
    let p1 = FourMomentum::new(10.0, 1.0, 0.0, 0.0);
    let p2 = FourMomentum::new(5.0, -1.0, 0.0, 0.0);

    let p_sum = p1 + p2;
    assert_eq!(p_sum.e(), 15.0);
    assert_eq!(p_sum.px(), 0.0);

    let p_diff = p1 - p2;
    assert_eq!(p_diff.e(), 5.0);
    assert_eq!(p_diff.px(), 2.0);
}

#[test]
fn test_rapidity_and_pseudorapidity() {
    // E=10, pz=0 => rapidity = 0
    let p_rest = FourMomentum::new(10.0, 1.0, 0.0, 0.0);
    assert_eq!(p_rest.rapidity(), 0.0);
    assert!(p_rest.pseudorapidity().abs() < 1e-10); // Check magnitude near zero

    // Beam direction particle
    // E=10, pz=8
    let p_boost = FourMomentum::new(10.0, 0.0, 0.0, 8.0);
    // Rapidity should be positive
    assert!(p_boost.rapidity() > 0.0);
}

#[test]
fn test_lightcone_coordinates() {
    let p = FourMomentum::new(10.0, 0.0, 0.0, 5.0);
    assert_eq!(p.lightcone_plus(), 15.0); // 10+5
    assert_eq!(p.lightcone_minus(), 5.0); // 10-5
}

#[test]
fn test_boost_z() {
    let p = FourMomentum::at_rest(1.0); // m=1, E=1, p=0
    let beta = 0.6; // gamma = 1/0.8 = 1.25

    let p_boosted = p.boost_z(beta);
    // E' = gamma(E - beta*pz) = 1.25 * (1 - 0) = 1.25
    // pz' = gamma(pz - beta*E) = 1.25 * (0 - 0.6*1) = -0.75
    // NOTE: boost_z(beta) boosts to a frame moving with +z velocity beta?
    // Formula check:
    // If we boost TO a frame moving at +v, the particle appears to move at -v.
    // pz_new = gamma(pz - vE) = 1.25(0 - 0.6) = -0.75. Correct.

    assert!((p_boosted.e() - 1.25).abs() < 1e-10);
    assert!((p_boosted.pz() - -0.75).abs() < 1e-10);
    // Mass should be invariant
    assert!((p_boosted.invariant_mass() - 1.0).abs() < 1e-10);
}

// =============================================================================
// Hadron Tests
// =============================================================================

#[test]
fn test_hadron_properties() {
    let p = FourMomentum::new(5.0, 3.0, 4.0, 0.0); // E=5, px=3, py=4, pz=0
    // pt = sqrt(3^2+4^2) = 5. Mass should be 0 (lightlike) ideally, but here E^2 - p^2 = 25 - 25 = 0.
    let h = Hadron::new(211, p); // 211 = pi+

    assert_eq!(h.pdg_id(), 211);
    assert_eq!(h.energy(), 5.0);
    assert_eq!(h.pt(), 5.0);
    assert_eq!(h.mass(), 0.0); // Massless in this example
    assert_eq!(h.momentum(), p);
}

// =============================================================================
// LundParameters Tests
// =============================================================================

#[test]
fn test_lund_parameters_default() {
    let params = LundParameters::default();
    assert!((params.kappa() - 1.0).abs() < 1e-10);
    assert!(params.strange_suppression() > 0.0);
    assert!(params.strange_suppression() < 1.0);
}

#[test]
fn test_lund_parameters_custom() {
    let params = LundParameters::new(
        2.0, // kappa
        0.5, // a
        0.8, // b
        0.4, // sigma_pt
        0.2, // strange
        0.1, // diquark
        0.6, // vector
        0.3, // min mass
    );

    assert_eq!(params.kappa(), 2.0);
    assert_eq!(params.lund_a(), 0.5);
    assert_eq!(params.min_invariant_mass(), 0.3);
}
