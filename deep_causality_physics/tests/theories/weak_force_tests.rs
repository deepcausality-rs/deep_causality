/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2026" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Comprehensive tests for Weak Force — SU(2)_L gauge theory.
//!
//! Coverage includes:
//! - Physical constants (G_F, M_W, M_Z, θ_W)
//! - Propagators (charged and neutral current)
//! - Decay widths and lifetimes
//! - Weak isospin representations
//! - SU(2) generators (Pauli matrices)

use deep_causality_physics::theories::{
    WeakField, WeakFieldOps, WeakIsospin, pauli_matrices, su2_generators,
};
use deep_causality_physics::{FERMI_CONSTANT, HIGGS_VEV, SIN2_THETA_W, W_MASS, Z_MASS};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, Manifold, Simplex, SimplicialComplexBuilder};

// ============================================================================
// Test Helpers
// ============================================================================

fn create_weak_field() -> WeakField {
    let mut builder = SimplicialComplexBuilder::new(0);
    let _ = builder.add_simplex(Simplex::new(vec![0]));
    let complex = builder.build().expect("Failed to build complex");
    let data = CausalTensor::new(vec![0.0], vec![1]).unwrap();
    let base = Manifold::new(complex, data, 0).expect("Failed to create manifold");

    // SU(2): spacetime_dim=4, lie_algebra_dim=3
    let num_points = base.len();
    // Connection shape: [num_points, spacetime_dim, lie_dim]
    let conn = CausalTensor::zeros(&[num_points, 4, 3]);

    // Use constructor from WeakOps specifically for convention compliance
    WeakField::new_field(base, conn).expect("Failed to create WeakField")
}

// ============================================================================
// Physical Constants Tests
// ============================================================================

#[test]
fn test_fermi_constant() {
    // G_F ≈ 1.166 × 10⁻⁵ GeV⁻²
    assert!(
        (FERMI_CONSTANT - 1.1663787e-5).abs() < 1e-10,
        "Fermi constant should match PDG value"
    );
}

#[test]
fn test_w_boson_mass() {
    // M_W ≈ 80.4 GeV
    const {
        assert!(W_MASS > 80.0 && W_MASS < 81.0, "W mass should be ~80.4 GeV");
    }

    let weak = create_weak_field();
    assert_eq!(weak.w_mass(), W_MASS);
}

#[test]
fn test_z_boson_mass() {
    // M_Z ≈ 91.2 GeV
    const {
        assert!(Z_MASS > 91.0 && Z_MASS < 92.0, "Z mass should be ~91.2 GeV");
    }

    let weak = create_weak_field();
    assert_eq!(weak.z_mass(), Z_MASS);
}

#[test]
fn test_weinberg_angle() {
    // sin²θ_W ≈ 0.231
    const {
        assert!(
            SIN2_THETA_W > 0.23 && SIN2_THETA_W < 0.24,
            "sin²θ_W should be ~0.231"
        );
    }

    // Verify relationship: sin²θ_W ≈ 1 - (M_W/M_Z)²
    let computed = 1.0 - (W_MASS / Z_MASS).powi(2);
    assert!(
        (SIN2_THETA_W - computed).abs() < 0.01,
        "sin²θ_W should satisfy 1 - (M_W/M_Z)²"
    );
}

#[test]
fn test_higgs_vev() {
    // v ≈ 246 GeV
    const {
        assert!(
            HIGGS_VEV > 245.0 && HIGGS_VEV < 247.0,
            "Higgs VEV should be ~246 GeV"
        );
    }

    // Verify relationship: v = (√2 G_F)^(-1/2)
    let computed = 1.0 / (2.0_f64.sqrt() * FERMI_CONSTANT).sqrt();
    assert!(
        (HIGGS_VEV - computed).abs() < 1.0,
        "Higgs VEV should satisfy v = (√2 G_F)^(-1/2)"
    );
}

// ============================================================================
// Propagator Tests
// ============================================================================

#[test]
fn test_charged_current_propagator_low_energy() {
    // At q² << M_W², D_W(q²) ≈ -1/M_W²
    let prop = WeakField::charged_current_propagator(0.1).unwrap();
    let expected = -1.0 / (W_MASS * W_MASS);

    assert!(
        (prop - expected).abs() < 1e-6,
        "Low-energy W propagator should be ≈ -1/M_W²"
    );
}

#[test]
fn test_charged_current_propagator_on_shell_error() {
    // At q² = M_W², propagator diverges (on-shell)
    let result = WeakField::charged_current_propagator(W_MASS * W_MASS);
    assert!(result.is_err(), "On-shell W should return error");
}

#[test]
fn test_charged_current_propagator_invalid() {
    let result = WeakField::charged_current_propagator(f64::NAN);
    assert!(result.is_err(), "NaN momentum should return error");
}

#[test]
fn test_neutral_current_propagator_neutrino() {
    let nu = WeakIsospin::neutrino();
    let prop = WeakField::neutral_current_propagator(0.1, &nu).unwrap();

    // Should be non-zero for neutrino
    assert!(prop.abs() > 0.0, "Neutrino Z propagator should be non-zero");
}

#[test]
fn test_neutral_current_propagator_on_shell_error() {
    let nu = WeakIsospin::neutrino();
    let result = WeakField::neutral_current_propagator(Z_MASS * Z_MASS, &nu);
    assert!(result.is_err(), "On-shell Z should return error");
}

// ============================================================================
// Decay Width and Lifetime Tests
// ============================================================================

#[test]
fn test_weak_decay_width_positive_mass() {
    // Γ = G_F² m⁵ / (192 π³)
    let width = WeakField::weak_decay_width(1.0).unwrap();
    assert!(width > 0.0, "Decay width should be positive");
}

#[test]
fn test_weak_decay_width_invalid_mass() {
    let result = WeakField::weak_decay_width(-1.0);
    assert!(result.is_err(), "Negative mass should return error");

    let result = WeakField::weak_decay_width(0.0);
    assert!(result.is_err(), "Zero mass should return error");
}

#[test]
fn test_muon_lifetime() {
    // τ_μ ≈ 2.2 μs = 2.2 × 10⁻⁶ s
    let lifetime = WeakField::muon_lifetime();
    assert!(
        lifetime > 2.0e-6 && lifetime < 2.3e-6,
        "Muon lifetime should be ≈ 2.2 μs, got {}",
        lifetime
    );
}

#[test]
fn test_w_boson_width() {
    // Γ_W ≈ 2.1 GeV
    let width = WeakField::w_boson_width();
    assert!(
        width > 1.5 && width < 3.0,
        "W width should be ≈ 2.1 GeV, got {}",
        width
    );
}

#[test]
fn test_z_boson_width() {
    // Γ_Z ≈ 2.5 GeV
    let width = WeakField::z_boson_width();
    assert!(
        width > 2.0 && width < 3.5,
        "Z width should be ≈ 2.5 GeV, got {}",
        width
    );
}

// ============================================================================
// Weak Isospin Tests
// ============================================================================

#[test]
fn test_lepton_doublet() {
    let lepton = WeakIsospin::lepton_doublet();
    assert_eq!(lepton.isospin, 0.5);
    assert_eq!(lepton.i3, -0.5);
    assert_eq!(lepton.hypercharge, -1.0);

    // Electric charge Q = I₃ + Y/2 = -0.5 + (-1)/2 = -1 (electron)
    let q = lepton.electric_charge();
    assert!((q - (-1.0)).abs() < 1e-10, "Electron charge should be -1");
}

#[test]
fn test_neutrino() {
    let nu = WeakIsospin::neutrino();
    assert_eq!(nu.isospin, 0.5);
    assert_eq!(nu.i3, 0.5);

    // Electric charge Q = I₃ + Y/2 = 0.5 + (-1)/2 = 0
    let q = nu.electric_charge();
    assert!(q.abs() < 1e-10, "Neutrino charge should be 0");
}

#[test]
fn test_up_quark() {
    let up = WeakIsospin::up_quark();
    assert_eq!(up.isospin, 0.5);
    assert_eq!(up.i3, 0.5);

    // Electric charge Q = I₃ + Y/2 = 0.5 + (1/3)/2 = 2/3
    let q = up.electric_charge();
    assert!(
        (q - 2.0 / 3.0).abs() < 1e-10,
        "Up quark charge should be +2/3"
    );
}

#[test]
fn test_down_quark() {
    let down = WeakIsospin::down_quark();
    assert_eq!(down.isospin, 0.5);
    assert_eq!(down.i3, -0.5);

    // Electric charge Q = I₃ + Y/2 = -0.5 + (1/3)/2 = -1/3
    let q = down.electric_charge();
    assert!(
        (q - (-1.0 / 3.0)).abs() < 1e-10,
        "Down quark charge should be -1/3"
    );
}

#[test]
fn test_right_handed_electron() {
    let e_r = WeakIsospin::right_handed(-1.0);
    assert_eq!(e_r.isospin, 0.0);
    assert_eq!(e_r.i3, 0.0);
    assert_eq!(e_r.hypercharge, -2.0);

    // Q = 0 + (-2)/2 = -1
    let q = e_r.electric_charge();
    assert!(
        (q - (-1.0)).abs() < 1e-10,
        "Right-handed e charge should be -1"
    );
}

#[test]
fn test_vector_axial_couplings() {
    let nu = WeakIsospin::neutrino();
    let g_v = nu.vector_coupling();
    let g_a = nu.axial_coupling();

    // For neutrino: g_V = I₃, g_A = I₃ = 0.5
    assert!((g_a - 0.5).abs() < 1e-10, "Neutrino g_A should be 0.5");
    // g_V = I₃ - 2Q sin²θ_W = 0.5 - 0 = 0.5
    assert!((g_v - 0.5).abs() < 1e-10, "Neutrino g_V should be 0.5");
}

#[test]
fn test_isospin_constraint() {
    // I₃ must satisfy |I₃| ≤ I
    let result = WeakIsospin::new(0.5, 1.0, 0.0);
    assert!(result.is_err(), "|I₃| > I should return error");
}

// ============================================================================
// SU(2) Generator Tests
// ============================================================================

#[test]
fn test_pauli_matrices() {
    let pauli = pauli_matrices();

    // σ₃ should be diagonal: (1,0), (0,-1)
    assert_eq!(pauli[2][0], (1.0, 0.0)); // σ₃[0,0] = 1
    assert_eq!(pauli[2][3], (-1.0, 0.0)); // σ₃[1,1] = -1
}

#[test]
fn test_su2_generators() {
    let generators = su2_generators();

    // T_a = σ_a / 2, so T₃[0,0] = 0.5
    assert_eq!(generators[2][0], (0.5, 0.0));
    assert_eq!(generators[2][3], (-0.5, 0.0));
}

// ============================================================================
// Gauge Field Structure Tests
// ============================================================================

#[test]
fn test_weak_field_is_non_abelian() {
    let weak = create_weak_field();
    assert!(!weak.is_abelian(), "SU(2) should be non-abelian");
}

#[test]
fn test_weak_field_west_coast() {
    let weak = create_weak_field();
    assert!(
        weak.is_west_coast(),
        "Weak field should use West Coast signature"
    );
}
