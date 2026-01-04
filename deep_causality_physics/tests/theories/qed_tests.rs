/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_multivector::{CausalMultiVector, Metric};
use deep_causality_physics::QED;

// =============================================================================
// QED Construction Tests
// =============================================================================

#[test]
fn test_qed_from_components_valid() {
    let qed = QED::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0);
    assert!(qed.is_ok());

    let field = qed.unwrap();
    assert_eq!(field.metric(), Metric::Euclidean(3));
}

#[test]
fn test_qed_from_fields_valid() {
    let e = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(
        vec![0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();

    let qed = QED::from_fields(e, b);
    assert!(qed.is_ok());
}

#[test]
fn test_qed_from_fields_metric_mismatch() {
    let e = CausalMultiVector::new(
        vec![0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        Metric::Euclidean(3),
    )
    .unwrap();
    let b = CausalMultiVector::new(vec![0.0, 0.0, 1.0, 0.0], Metric::Euclidean(2)).unwrap();

    let qed = QED::from_fields(e, b);
    assert!(qed.is_err());
}

#[test]
fn test_qed_plane_wave_valid() {
    let qed = QED::plane_wave(1.0, 0);
    assert!(qed.is_ok());

    let field = qed.unwrap();
    assert!(field.is_radiation_field()); // E ⟂ B
    assert!(field.is_null_field()); // |E| = |B|
}

#[test]
fn test_qed_plane_wave_invalid_polarization() {
    let qed = QED::plane_wave(1.0, 5);
    assert!(qed.is_err());
}

#[test]
fn test_qed_plane_wave_invalid_amplitude() {
    let qed = QED::plane_wave(f64::NAN, 0);
    assert!(qed.is_err());
}

#[test]
fn test_qed_default() {
    let qed = QED::default();
    assert_eq!(qed.metric(), Metric::Euclidean(3));

    // Zero fields should have zero energy
    let energy = qed.energy_density().unwrap();
    assert!(energy.abs() < 1e-10);
}

// =============================================================================
// Energy Density Tests
// =============================================================================

#[test]
fn test_qed_energy_density_unit_fields() {
    // E = (1, 0, 0), B = (0, 1, 0)
    // u = (1 + 1) / 2 = 1.0
    let qed = QED::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();
    let energy = qed.energy_density().unwrap();
    assert!((energy - 1.0).abs() < 1e-10, "Expected 1.0, got {}", energy);
}

#[test]
fn test_qed_energy_density_zero_field() {
    let qed = QED::from_components(0.0, 0.0, 0.0, 0.0, 0.0, 0.0).unwrap();
    let energy = qed.energy_density().unwrap();
    assert!(energy.abs() < 1e-10);
}

#[test]
fn test_qed_energy_density_electric_only() {
    // E = (2, 0, 0), B = (0, 0, 0)
    // u = (4 + 0) / 2 = 2.0
    let qed = QED::from_components(2.0, 0.0, 0.0, 0.0, 0.0, 0.0).unwrap();
    let energy = qed.energy_density().unwrap();
    assert!((energy - 2.0).abs() < 1e-10, "Expected 2.0, got {}", energy);
}

// =============================================================================
// Lagrangian Density Tests
// =============================================================================

#[test]
fn test_qed_lagrangian_density_plane_wave() {
    // Plane wave: |E| = |B|, so L = (E² - B²)/2 = 0
    let qed = QED::plane_wave(1.0, 0).unwrap();
    let lagrangian = qed.lagrangian_density().unwrap();
    assert!(
        lagrangian.abs() < 1e-10,
        "Plane wave should have L = 0, got {}",
        lagrangian
    );
}

#[test]
fn test_qed_lagrangian_density_electric_dominated() {
    // E = (2, 0, 0), B = (0, 1, 0)
    // L = (4 - 1) / 2 = 1.5
    let qed = QED::from_components(2.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();
    let lagrangian = qed.lagrangian_density().unwrap();
    assert!(
        (lagrangian - 1.5).abs() < 1e-10,
        "Expected 1.5, got {}",
        lagrangian
    );
}

#[test]
fn test_qed_lagrangian_density_magnetic_dominated() {
    // E = (1, 0, 0), B = (0, 2, 0)
    // L = (1 - 4) / 2 = -1.5
    let qed = QED::from_components(1.0, 0.0, 0.0, 0.0, 2.0, 0.0).unwrap();
    let lagrangian = qed.lagrangian_density().unwrap();
    assert!(
        (lagrangian - (-1.5)).abs() < 1e-10,
        "Expected -1.5, got {}",
        lagrangian
    );
}

// =============================================================================
// Poynting Vector Tests
// =============================================================================

#[test]
fn test_qed_poynting_vector_orthogonal_fields() {
    // E along x, B along y → S along z (as bivector xy)
    let qed = QED::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();
    let poynting = qed.poynting_vector();
    assert!(poynting.is_ok());

    let s = poynting.unwrap();
    assert!(!s.data().is_empty());
}

#[test]
fn test_qed_poynting_vector_zero_field() {
    let qed = QED::default();
    let poynting = qed.poynting_vector().unwrap();

    // Zero fields should have zero Poynting vector
    for val in poynting.data() {
        assert!(val.abs() < 1e-10);
    }
}

// =============================================================================
// Field Invariants Tests
// =============================================================================

#[test]
fn test_qed_field_invariant_plane_wave() {
    // For plane wave: |E| = |B|, so F_μν F^μν = 2(B² - E²) = 0
    let qed = QED::plane_wave(1.0, 0).unwrap();
    let invariant = qed.field_invariant().unwrap();
    assert!(invariant.abs() < 1e-10, "Expected 0, got {}", invariant);
}

#[test]
fn test_qed_field_invariant_electric_only() {
    // E only: F_μν F^μν = 2(0 - E²) = -2E²
    let qed = QED::from_components(1.0, 0.0, 0.0, 0.0, 0.0, 0.0).unwrap();
    let invariant = qed.field_invariant().unwrap();
    assert!(
        (invariant - (-2.0)).abs() < 1e-10,
        "Expected -2, got {}",
        invariant
    );
}

#[test]
fn test_qed_field_invariant_magnetic_only() {
    // B only: F_μν F^μν = 2(B² - 0) = 2B²
    let qed = QED::from_components(0.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
    let invariant = qed.field_invariant().unwrap();
    assert!(
        (invariant - 2.0).abs() < 1e-10,
        "Expected 2, got {}",
        invariant
    );
}

#[test]
fn test_qed_dual_invariant_plane_wave() {
    // Plane wave: E ⟂ B, so E·B = 0, dual = -4(E·B) = 0
    let qed = QED::plane_wave(1.0, 0).unwrap();
    let dual = qed.dual_invariant().unwrap();
    assert!(dual.abs() < 1e-10, "Expected 0, got {}", dual);
}

#[test]
fn test_qed_dual_invariant_parallel_fields() {
    // E ∥ B: E = B = (1, 0, 0)
    // E·B = 1, dual = -4(1) = -4
    let qed = QED::from_components(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
    let dual = qed.dual_invariant().unwrap();
    assert!((dual - (-4.0)).abs() < 1e-10, "Expected -4, got {}", dual);
}

// =============================================================================
// Field Classification Tests
// =============================================================================

#[test]
fn test_qed_is_radiation_field_true() {
    // Plane wave: E ⟂ B
    let qed = QED::plane_wave(1.0, 0).unwrap();
    assert!(qed.is_radiation_field());
}

#[test]
fn test_qed_is_radiation_field_false() {
    // Parallel E and B
    let qed = QED::from_components(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
    assert!(!qed.is_radiation_field());
}

#[test]
fn test_qed_is_null_field_true() {
    // Plane wave: |E| = |B|
    let qed = QED::plane_wave(1.0, 0).unwrap();
    assert!(qed.is_null_field());
}

#[test]
fn test_qed_is_null_field_false() {
    // |E| ≠ |B|
    let qed = QED::from_components(2.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();
    assert!(!qed.is_null_field());
}

// =============================================================================
// Intensity and Momentum Tests
// =============================================================================

#[test]
fn test_qed_intensity_unit_fields() {
    let qed = QED::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();
    let intensity = qed.intensity();
    assert!(intensity.is_ok());

    let i = intensity.unwrap();
    assert!(i > 0.0, "Intensity should be positive");
}

#[test]
fn test_qed_momentum_density_equals_poynting() {
    // In natural units, g = S
    let qed = QED::from_components(1.0, 0.0, 0.0, 0.0, 1.0, 0.0).unwrap();

    let poynting = qed.poynting_vector().unwrap();
    let momentum = qed.momentum_density().unwrap();

    // Should be equal
    for (s, g) in poynting.data().iter().zip(momentum.data().iter()) {
        assert!((s - g).abs() < 1e-10);
    }
}
