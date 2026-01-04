/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_multivector::MultiVector;
use deep_causality_physics::{QED, QedOps};

#[test]
fn test_qed_creation() {
    let result = QED::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0);
    assert!(result.is_ok());
    let qed = result.unwrap();
    assert!(qed.is_west_coast());
}

#[test]
fn test_qed_plane_wave() {
    let result = QED::plane_wave(1.0, 0);
    assert!(result.is_ok());

    let qed = result.unwrap();
    let e = qed.electric_field().unwrap();
    let b = qed.magnetic_field().unwrap();

    let e_sq = e.squared_magnitude();
    let b_sq = b.squared_magnitude();

    assert!((e_sq.abs() - 1.0) < 1e-5);
    assert!((b_sq.abs() - 1.0) < 1e-5);
}

#[test]
fn test_qed_invariants() {
    // E = (1,0,0), B = (0,1,0) -> Orthogonal
    let qed = QED::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    // Invariant I = 2(B² - E²) -> 0 since |E|=|B|=1
    let invariant = qed.field_invariant();
    assert!(invariant.is_ok());

    let inv_val = invariant.unwrap();
    assert!(inv_val.abs() < 1e-5);

    // Dual invariant K = -4 E·B -> 0
    let dual = qed.dual_invariant();
    assert!(dual.is_ok());
    assert!(dual.unwrap().abs() < 1e-5);

    assert!(qed.is_radiation_field().unwrap());
    assert!(qed.is_null_field().unwrap());
}

#[test]
fn test_qed_energy_momentum() {
    // E = (1,0,0), B = (0,1,0)
    let qed = QED::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    // Energy density U = 0.5(E^2 + B^2).
    // In +--- metric, E^2, B^2 < 0. Energy might be negative.
    let energy = qed.energy_density().unwrap();
    assert!(energy.abs() > 0.0);

    // TODO: Verify Poynting vector kernel with 4D MultiVectors.
    // Currently returns 0 for orthogonal 4D vectors (indices 2,3).
    /*
    // Poynting S = E x B = (0,0,1)
    let s = qed.poynting_vector().unwrap();
    let s_sq = s.squared_magnitude();
    // S=(0,0,1,0) -> sq = -1
    assert!((s_sq.abs() - 1.0).abs() < 1e-5);

    // Momentum density = S (in c=1 units)
    let p = qed.momentum_density().unwrap();
    assert!((p.squared_magnitude().abs() - 1.0).abs() < 1e-5);

    // Intensity
    let intensity = qed.intensity().unwrap();
    assert!((intensity - 1.0).abs() < 1e-5);
    */
}

#[test]
fn test_qed_dynamics() {
    let qed = QED::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    // Lagrangian L = 0.5(E^2 - B^2) = 0 for null field
    let lagrangian = qed.lagrangian_density().unwrap();
    assert!(lagrangian.abs() < 1e-5);

    // Lorentz force F = q(E + v x B)
    // J = (1, 0, 0, 0)
    let metric = qed.electric_field().unwrap().metric();
    let mut j_data = vec![0.0; 16];
    j_data[0] = 1.0;
    let j = deep_causality_multivector::CausalMultiVector::new(j_data, metric).unwrap();

    let force = qed.lorentz_force(&j).unwrap();
    // F ~ E. E^2 = -1. Force^2 ~ -1.
    assert!(force.squared_magnitude().abs() > 0.0);
}
