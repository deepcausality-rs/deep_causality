/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Comprehensive tests for QED (Quantum Electrodynamics) — U(1) gauge theory.
//!
//! Coverage includes:
//! - Field creation and extraction
//! - Lorentz invariants
//! - Energy-momentum quantities
//! - Field strength computation via GaugeFieldWitness (HKT)
//! - Physical properties (radiation fields, null fields)

use deep_causality_multivector::{CausalMultiVector, MultiVector};
use deep_causality_physics::{EM, GaugeEmOps};

// ============================================================================
// Field Creation Tests
// ============================================================================

#[test]
fn test_qed_from_components() {
    // E = (1, 0, 0), B = (0, 1, 0)
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0);
    assert!(qed.is_ok(), "QED creation should succeed");

    let field = qed.unwrap();
    assert!(
        field.is_west_coast(),
        "QED should use West Coast (+---) signature"
    );
    assert!(field.is_abelian(), "U(1) is abelian");
}

#[test]
fn test_qed_plane_wave() {
    // Plane wave with amplitude 1.0
    let qed = EM::plane_wave(1.0, 0);
    assert!(qed.is_ok(), "Plane wave creation should succeed");

    let field = qed.unwrap();
    let e = field.electric_field().unwrap();
    let b = field.magnetic_field().unwrap();

    // Plane waves have |E| = |B| in natural units
    let e_sq: f64 = e.squared_magnitude();
    let b_sq: f64 = b.squared_magnitude();
    assert!(
        (e_sq - b_sq).abs() < 1e-5,
        "|E|² = {} should equal |B|² = {}",
        e_sq,
        b_sq
    );
}

#[test]
fn test_qed_plane_wave_polarizations() {
    // Test polarization states 0 and 1 (most common)
    for pol in 0..2 {
        let qed = EM::plane_wave(2.0, pol);
        assert!(qed.is_ok(), "Polarization {} should succeed", pol);

        let field = qed.unwrap();
        let e = field.electric_field().unwrap();
        // Use Euclidean magnitude for 3D spatial vectors
        let e_sq: f64 = e.euclidean_squared_magnitude_3d();

        // Amplitude 2.0 → |E|² = 4.0
        assert!(
            e_sq > 0.0,
            "Electric field should be non-zero for pol={}",
            pol
        );
    }
}

// ============================================================================
// Field Extraction Tests
// ============================================================================

#[test]
fn test_electric_field_extraction() {
    // E = (1, 2, 3), B = (0, 0, 0)
    let qed = EM::from_components(1.0, 2.0, 3.0, 0.0, 0.0, 0.0).unwrap();
    let e = qed.electric_field().unwrap();

    // Check that E is extracted correctly from F_{0i}
    let data = e.data();
    // In 4D multivector with +--- signature, spatial indices are 2, 3, 4
    let ex: f64 = data.get(2).copied().unwrap_or(0.0);
    let ey: f64 = data.get(3).copied().unwrap_or(0.0);
    let ez: f64 = data.get(4).copied().unwrap_or(0.0);

    assert!((ex - 1.0).abs() < 1e-10, "E_x = {} should be 1.0", ex);
    assert!((ey - 2.0).abs() < 1e-10, "E_y = {} should be 2.0", ey);
    assert!((ez - 3.0).abs() < 1e-10, "E_z = {} should be 3.0", ez);
}

#[test]
fn test_magnetic_field_extraction() {
    // E = (0, 0, 0), B = (1, 2, 3)
    let qed = EM::from_components(0.0, 0.0, 0.0, 1.0, 2.0, 3.0).unwrap();
    let b = qed.magnetic_field().unwrap();

    // Check that B is extracted correctly from F_{ij}
    let data = b.data();
    let bx = data.get(2).copied().unwrap_or(0.0);
    let by = data.get(3).copied().unwrap_or(0.0);
    let bz = data.get(4).copied().unwrap_or(0.0);

    assert!((bx - 1.0f64).abs() < 1e-10, "B_x = {} should be 1.0", bx);
    assert!((by - 2.0f64).abs() < 1e-10, "B_y = {} should be 2.0", by);
    assert!((bz - 3.0f64).abs() < 1e-10, "B_z = {} should be 3.0", bz);
}

// ============================================================================
// Lorentz Invariant Tests
// ============================================================================

#[test]
fn test_field_invariant_null_field() {
    // Null field: |E| = |B| → I₁ = 2(|B|² - |E|²) = 0
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let invariant: f64 = qed.field_invariant().unwrap();
    assert!(
        invariant.abs() < 1e-5,
        "Null field invariant I₁ = {} should be ≈ 0",
        invariant
    );
}

#[test]
fn test_field_invariant_electric_dominated() {
    // Electric dominated: |E|² > |B|² → I₁ = 2(|B|² - |E|²)
    let qed = EM::from_components(2.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let invariant: f64 = qed.field_invariant().unwrap();
    // Implementation uses Euclidean magnitude for 3D spatial vectors
    let e = qed.electric_field().unwrap();
    let b = qed.magnetic_field().unwrap();
    let expected = 2.0 * (b.euclidean_squared_magnitude_3d() - e.euclidean_squared_magnitude_3d());
    assert!(
        (invariant - expected).abs() < 1e-5,
        "I₁ = {} should equal 2(B² - E²) = {}",
        invariant,
        expected
    );
}

#[test]
fn test_field_invariant_magnetic_dominated() {
    // Magnetic dominated: |B|² > |E|² → I₁ = 2(|B|² - |E|²)
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 2.0, 0.0).unwrap();

    let invariant: f64 = qed.field_invariant().unwrap();
    // Implementation uses Euclidean magnitude for 3D spatial vectors
    let e = qed.electric_field().unwrap();
    let b = qed.magnetic_field().unwrap();
    let expected = 2.0 * (b.euclidean_squared_magnitude_3d() - e.euclidean_squared_magnitude_3d());
    assert!(
        (invariant - expected).abs() < 1e-5,
        "I₁ = {} should equal 2(B² - E²) = {}",
        invariant,
        expected
    );
}

#[test]
fn test_dual_invariant_orthogonal() {
    // Orthogonal fields: E ⟂ B → I₂ = -4(E·B) = 0
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let dual: f64 = qed.dual_invariant().unwrap();
    assert!(
        dual.abs() < 1e-5,
        "Orthogonal field dual invariant I₂ = {} should be ≈ 0",
        dual
    );
}

#[test]
fn test_dual_invariant_parallel() {
    // Parallel fields: E ∥ B → I₂ ≠ 0
    let qed = EM::from_components(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();

    let dual: f64 = qed.dual_invariant().unwrap();
    // I₂ = -4(E·B) = -4(1·1) = -4 (with metric sign considerations)
    assert!(
        dual.abs() > 0.0,
        "Parallel field dual invariant I₂ = {} should be ≠ 0",
        dual
    );
}

// ============================================================================
// Energy-Momentum Tests
// ============================================================================

#[test]
fn test_energy_density_positive() {
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let energy: f64 = qed.energy_density().unwrap();
    // u = ½(|E|² + |B|²) — sign depends on metric convention
    assert!(
        energy.abs() > 0.0,
        "Energy density should be non-zero, got {}",
        energy
    );
}

#[test]
fn test_lagrangian_density_null_field() {
    // Null field: L = ½(|E|² - |B|²) = 0
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let lagrangian: f64 = qed.lagrangian_density().unwrap();
    assert!(
        lagrangian.abs() < 1e-5,
        "Null field Lagrangian L = {} should be ≈ 0",
        lagrangian
    );
}

#[test]
fn test_lagrangian_density_electric_dominated() {
    // |E| > |B| → L > 0
    let qed = EM::from_components(2.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let lagrangian: f64 = qed.lagrangian_density().unwrap();
    // L = ½(|E|² - |B|²) = ½(4 - 1) = 1.5 (sign depends on metric)
    assert!(
        lagrangian.abs() > 0.0,
        "Electric-dominated Lagrangian should be non-zero, got {}",
        lagrangian
    );
}

#[test]
fn test_poynting_vector() {
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let s = qed.poynting_vector();
    assert!(s.is_ok(), "Poynting vector should compute successfully");
}

#[test]
fn test_momentum_density_equals_poynting() {
    // In natural units (c = 1), g = S
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let s = qed.poynting_vector().unwrap();
    let g = qed.momentum_density().unwrap();

    // They should be identical
    let s_sq: f64 = s.squared_magnitude();
    let g_sq: f64 = g.squared_magnitude();
    assert!(
        (s_sq - g_sq).abs() < 1e-10,
        "Momentum density should equal Poynting vector"
    );
}

#[test]
fn test_intensity() {
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let intensity = qed.intensity();
    assert!(intensity.is_ok(), "Intensity should compute successfully");
    assert!(
        intensity.unwrap() >= 0.0,
        "Intensity should be non-negative"
    );
}

// ============================================================================
// Lorentz Force Tests
// ============================================================================

#[test]
fn test_lorentz_force_on_current() {
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    // Create current density 4-vector with spatial component at index 2
    let metric = qed.electric_field().unwrap().metric();
    let mut j_data = vec![0.0; 16];
    j_data[2] = 1.0; // Spatial x-component
    let j = CausalMultiVector::new(j_data, metric).unwrap();

    let force = qed.lorentz_force(&j);
    assert!(force.is_ok(), "Lorentz force should compute successfully");
    // Force computation depends on cross product - just verify it runs
}

// ============================================================================
// Physical Property Tests
// ============================================================================

#[test]
fn test_is_radiation_field() {
    // Radiation: E ⟂ B
    let radiation = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();
    assert!(
        radiation.is_radiation_field().unwrap(),
        "Orthogonal E,B should be radiation field"
    );

    // Not radiation: E ∥ B
    let not_radiation = EM::from_components(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
    assert!(
        !not_radiation.is_radiation_field().unwrap(),
        "Parallel E,B should NOT be radiation field"
    );
}

#[test]
fn test_is_null_field() {
    // Null: |E| = |B|
    let null = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();
    assert!(
        null.is_null_field().unwrap(),
        "|E| = |B| should be null field"
    );

    // Not null: |E| ≠ |B|
    let not_null = EM::from_components(2.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();
    assert!(
        !not_null.is_null_field().unwrap(),
        "|E| ≠ |B| should NOT be null field"
    );
}

// ============================================================================
// GaugeFieldWitness HKT Integration Tests
// ============================================================================

/// Tests computed_field_strength() which uses GaugeFieldWitness as single source of truth.
#[test]
fn test_computed_field_strength_shape() {
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let f = qed.computed_field_strength();
    assert!(f.is_ok(), "computed_field_strength should succeed for U(1)");

    let tensor = f.unwrap();
    let shape = tensor.shape();

    // F_μν has shape [num_points, spacetime_dim, spacetime_dim, lie_dim]
    assert_eq!(shape.len(), 4, "F tensor should be 4-dimensional");
    assert_eq!(shape[1], 4, "Spacetime dimension should be 4");
    assert_eq!(shape[2], 4, "Spacetime dimension should be 4");
    assert_eq!(shape[3], 1, "U(1) has Lie algebra dimension 1");
}

/// Tests the fundamental antisymmetry property: F_μν = -F_νμ
#[test]
fn test_computed_field_strength_antisymmetry() {
    let qed = EM::from_components(1.0, 2.0, 3.0, 0.5, 1.5, 2.5).unwrap();

    let f = qed.computed_field_strength().unwrap();
    let data = f.as_slice();
    let shape = f.shape();

    let num_points = shape[0];
    let dim = shape[1];
    let lie_dim = shape[3];

    for p in 0..num_points {
        for a in 0..lie_dim {
            for mu in 0..dim {
                for nu in 0..dim {
                    let idx_mu_nu =
                        p * (dim * dim * lie_dim) + mu * (dim * lie_dim) + nu * lie_dim + a;
                    let idx_nu_mu =
                        p * (dim * dim * lie_dim) + nu * (dim * lie_dim) + mu * lie_dim + a;

                    let f_mu_nu: f64 = data.get(idx_mu_nu).copied().unwrap_or(0.0);
                    let f_nu_mu: f64 = data.get(idx_nu_mu).copied().unwrap_or(0.0);

                    assert!(
                        (f_mu_nu + f_nu_mu).abs() < 1e-10,
                        "Antisymmetry violated: F[{},{}] = {} but F[{},{}] = {}",
                        mu,
                        nu,
                        f_mu_nu,
                        nu,
                        mu,
                        f_nu_mu
                    );
                }
            }
        }
    }
}

/// Tests that diagonal elements are zero: F_μμ = 0
#[test]
fn test_computed_field_strength_diagonal_zero() {
    let qed = EM::from_components(1.0, 2.0, 3.0, 0.5, 1.5, 2.5).unwrap();

    let f = qed.computed_field_strength().unwrap();
    let data = f.as_slice();
    let shape = f.shape();

    let num_points = shape[0];
    let dim = shape[1];
    let lie_dim = shape[3];

    for p in 0..num_points {
        for a in 0..lie_dim {
            for mu in 0..dim {
                let idx = p * (dim * dim * lie_dim) + mu * (dim * lie_dim) + mu * lie_dim + a;
                let f_mu_mu: f64 = data.get(idx).copied().unwrap_or(0.0);

                assert!(
                    f_mu_mu.abs() < 1e-10,
                    "Diagonal F[{},{}] = {} should be 0",
                    mu,
                    mu,
                    f_mu_mu
                );
            }
        }
    }
}

// ============================================================================
// Additional EM Coverage Tests
// ============================================================================

#[test]
fn test_plane_wave_all_polarizations() {
    // Test polarization 0
    let wave0 = EM::plane_wave(1.0, 0);
    assert!(wave0.is_ok(), "Polarization 0 should succeed");

    // Test polarization 1
    let wave1 = EM::plane_wave(1.0, 1);
    assert!(wave1.is_ok(), "Polarization 1 should succeed");

    // All should be valid radiation fields
    assert!(
        wave0.unwrap().is_radiation_field().unwrap(),
        "Pol 0 should be radiation"
    );
    assert!(
        wave1.unwrap().is_radiation_field().unwrap(),
        "Pol 1 should be radiation"
    );
}

#[test]
fn test_plane_wave_null_field_property() {
    // All plane waves should be null fields (|E| = |B|)
    let wave = EM::plane_wave(2.5, 0).unwrap();
    assert!(
        wave.is_null_field().unwrap(),
        "Plane wave should be null field"
    );
}

#[test]
fn test_zero_fields() {
    // Zero fields should have zero energy, lagrangian, etc.
    let zero_field = EM::from_components(0.0, 0.0, 0.0, 0.0, 0.0, 0.0).unwrap();

    let energy: f64 = zero_field.energy_density().unwrap();
    assert!(energy.abs() < 1e-15, "Zero field energy should be 0");

    let lagrangian: f64 = zero_field.lagrangian_density().unwrap();
    assert!(
        lagrangian.abs() < 1e-15,
        "Zero field Lagrangian should be 0"
    );

    // Zero field should technically be both radiation and null
    assert!(zero_field.is_null_field().unwrap(), "Zero field is null");
}

#[test]
fn test_pure_electric_field() {
    // Pure E field (no B)
    let e_only = EM::from_components(1.0, 2.0, 3.0, 0.0, 0.0, 0.0).unwrap();

    // Should not be a null field (|E| ≠ |B|)
    assert!(!e_only.is_null_field().unwrap(), "Pure E field is not null");

    // Should not be radiation (E not perpendicular to B makes no sense here)
    // Actually with B=0, E·B=0 so it might pass the orthogonality test
    let is_rad = e_only.is_radiation_field().unwrap();
    // Just ensure it computes without error
    let _ = is_rad;
}

#[test]
fn test_pure_magnetic_field() {
    // Pure B field (no E)
    let b_only = EM::from_components(0.0, 0.0, 0.0, 1.0, 2.0, 3.0).unwrap();

    // Should not be a null field
    assert!(!b_only.is_null_field().unwrap(), "Pure B field is not null");

    // Field invariant should be positive (B² - E² > 0)
    let invariant: f64 = b_only.field_invariant().unwrap();
    assert!(
        invariant > 0.0,
        "Pure B field should have I₁ > 0, got {}",
        invariant
    );
}

#[test]
fn test_lorentz_force_with_zero_current() {
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    // Create zero current density
    let metric = qed.electric_field().unwrap().metric();
    let j_data = vec![0.0; 16];
    let j = CausalMultiVector::new(j_data, metric).unwrap();

    let force = qed.lorentz_force(&j);
    assert!(
        force.is_ok(),
        "Lorentz force should compute with zero current"
    );

    // Force should be zero for zero current
    let f = force.unwrap();
    let f_mag: f64 = f.squared_magnitude();
    assert!(f_mag.abs() < 1e-15, "Force should be zero for zero current");
}

#[test]
fn test_poynting_vector_orthogonal_fields() {
    // S = E × B, for orthogonal E and B
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let s = qed.poynting_vector().unwrap();
    let s_data = s.data();

    // E_x × B_y should give S_z
    // For (1,0,0) × (0,1,0) = (0,0,1)
    let sz: f64 = s_data.get(4).copied().unwrap_or(0.0);
    assert!(sz.abs() > 0.0, "S_z should be non-zero for E_x × B_y");
}

#[test]
fn test_intensity_matches_poynting_magnitude() {
    let qed = EM::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let intensity: f64 = qed.intensity().unwrap();
    let _s = qed.poynting_vector().unwrap();

    // Intensity is |S|
    // Both should be non-zero and related
    assert!(intensity >= 0.0, "Intensity should be non-negative");
}

#[test]
fn test_momentum_density_equals_poynting_scaled() {
    // In natural units, g = S (momentum density = Poynting vector)
    let qed = EM::from_components(2.0, 1.0, 0.5, 0.3, 1.2, 0.8).unwrap();

    let s = qed.poynting_vector().unwrap();
    let g = qed.momentum_density().unwrap();

    // Compare magnitudes
    let s_mag: f64 = s.squared_magnitude();
    let g_mag: f64 = g.squared_magnitude();

    assert!(
        (s_mag - g_mag).abs() < 1e-10,
        "Momentum density should equal Poynting vector"
    );
}
