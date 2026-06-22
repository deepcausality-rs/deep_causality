/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_physics::FourMomentum;

#[test]
fn test_four_momentum_creation_and_accessors() {
    let p = FourMomentum::<f64>::new(10.0, 1.0, 2.0, 3.0);
    assert_eq!(p.e(), 10.0);
    assert_eq!(p.px(), 1.0);
    assert_eq!(p.py(), 2.0);
    assert_eq!(p.pz(), 3.0);
}

#[test]
fn test_four_momentum_at_rest() {
    let p = FourMomentum::<f64>::at_rest(5.0);
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
    let p = FourMomentum::<f64>::from_mass_and_momentum(mass, px, py, pz);
    assert!((p.e() - 5.0).abs() < 1e-10);
    assert!((p.invariant_mass() - 3.0).abs() < 1e-10);
}

#[test]
fn test_four_momentum_math() {
    let p1 = FourMomentum::<f64>::new(10.0, 1.0, 0.0, 0.0);
    let p2 = FourMomentum::<f64>::new(5.0, -1.0, 0.0, 0.0);

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
    let p_rest = FourMomentum::<f64>::new(10.0, 1.0, 0.0, 0.0);
    assert_eq!(p_rest.rapidity(), 0.0);
    assert!(p_rest.pseudorapidity().abs() < 1e-10); // Check magnitude near zero

    // Beam direction particle
    // E=10, pz=8
    let p_boost = FourMomentum::<f64>::new(10.0, 0.0, 0.0, 8.0);
    // Rapidity should be positive
    assert!(p_boost.rapidity() > 0.0);
}

#[test]
fn test_lightcone_coordinates() {
    let p = FourMomentum::<f64>::new(10.0, 0.0, 0.0, 5.0);
    assert_eq!(p.lightcone_plus(), 15.0); // 10+5
    assert_eq!(p.lightcone_minus(), 5.0); // 10-5
}

#[test]
fn test_boost_z() {
    let p = FourMomentum::<f64>::at_rest(1.0); // m=1, E=1, p=0
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

#[test]
fn test_four_momentum_default() {
    // nuclear/mod.rs:172-179
    let p = FourMomentum::<f64>::default();
    assert_eq!(p.e(), 0.0);
    assert_eq!(p.px(), 0.0);
    assert_eq!(p.py(), 0.0);
    assert_eq!(p.pz(), 0.0);
}

#[test]
fn test_four_momentum_transverse_mass() {
    // nuclear/mod.rs:255-259. m^2 = E^2 - |p|^2; pt^2 = px^2 + py^2.
    // E=10, px=3, py=4, pz=0 => m^2 = 100 - 25 = 75, pt^2 = 25 => mT = sqrt(100) = 10.
    let p = FourMomentum::<f64>::new(10.0, 3.0, 4.0, 0.0);
    assert!((p.transverse_mass() - 10.0).abs() < 1e-10);
}

#[test]
fn test_four_momentum_phi() {
    // nuclear/mod.rs:272-274. phi = atan2(py, px); px=1, py=1 => pi/4.
    let p = FourMomentum::<f64>::new(2.0, 1.0, 1.0, 0.0);
    assert!((p.phi() - std::f64::consts::FRAC_PI_4).abs() < 1e-10);
}

#[test]
fn test_four_momentum_rapidity_near_zero_denominator() {
    // nuclear/mod.rs:284-285. denom = E - pz near zero => early return 0.
    let p = FourMomentum::<f64>::new(5.0, 0.1, 0.0, 5.0); // E == pz
    assert_eq!(p.rapidity(), 0.0);
}

#[test]
fn test_four_momentum_pseudorapidity_near_zero_momentum() {
    // nuclear/mod.rs:297-298. |p| near zero => early return 0.
    let p = FourMomentum::<f64>::new(1.0, 0.0, 0.0, 0.0); // at rest, |p| = 0
    assert_eq!(p.pseudorapidity(), 0.0);
}
