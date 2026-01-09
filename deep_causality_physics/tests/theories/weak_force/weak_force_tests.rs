/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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

fn create_weak_field() -> WeakField<f64> {
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
    let prop = WeakField::<f64>::charged_current_propagator(0.1).unwrap();
    let expected = -1.0 / (W_MASS * W_MASS);

    assert!(
        (prop - expected).abs() < 1e-6,
        "Low-energy W propagator should be ≈ -1/M_W²"
    );
}

#[test]
fn test_charged_current_propagator_on_shell_error() {
    // At q² = M_W², propagator diverges (on-shell)
    let result = WeakField::<f64>::charged_current_propagator(W_MASS * W_MASS);
    assert!(result.is_err(), "On-shell W should return error");
}

#[test]
fn test_charged_current_propagator_invalid() {
    let result = WeakField::<f64>::charged_current_propagator(f64::NAN);
    assert!(result.is_err(), "NaN momentum should return error");
}

#[test]
fn test_neutral_current_propagator_neutrino() {
    let nu = WeakIsospin::neutrino();
    let prop = WeakField::<f64>::neutral_current_propagator(0.1, &nu).unwrap();

    // Should be non-zero for neutrino
    assert!(prop.abs() > 0.0, "Neutrino Z propagator should be non-zero");
}

#[test]
fn test_neutral_current_propagator_on_shell_error() {
    let nu = WeakIsospin::neutrino();
    let result = WeakField::<f64>::neutral_current_propagator(Z_MASS * Z_MASS, &nu);
    assert!(result.is_err(), "On-shell Z should return error");
}

// ============================================================================
// Decay Width and Lifetime Tests
// ============================================================================

#[test]
fn test_weak_decay_width_positive_mass() {
    // Γ = G_F² m⁵ / (192 π³)
    let width = WeakField::<f64>::weak_decay_width(1.0).unwrap();
    assert!(width > 0.0, "Decay width should be positive");
}

#[test]
fn test_weak_decay_width_invalid_mass() {
    let result = WeakField::<f64>::weak_decay_width(-1.0);
    assert!(result.is_err(), "Negative mass should return error");

    let result = WeakField::<f64>::weak_decay_width(0.0);
    assert!(result.is_err(), "Zero mass should return error");
}

#[test]
fn test_muon_lifetime() {
    // τ_μ ≈ 2.2 μs = 2.2 × 10⁻⁶ s
    let lifetime = WeakField::<f64>::muon_lifetime();
    assert!(
        lifetime > 2.0e-6 && lifetime < 2.3e-6,
        "Muon lifetime should be ≈ 2.2 μs, got {}",
        lifetime
    );
}

#[test]
fn test_w_boson_width() {
    // Γ_W ≈ 2.1 GeV
    let width = WeakField::<f64>::w_boson_width();
    assert!(
        width > 1.5 && width < 3.0,
        "W width should be ≈ 2.1 GeV, got {}",
        width
    );
}

#[test]
fn test_z_boson_width() {
    // Γ_Z ≈ 2.5 GeV
    let width = WeakField::<f64>::z_boson_width();
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

// ============================================================================
// Additional Coverage Tests
// ============================================================================

#[test]
fn test_weak_isospin_new_ok_path() {
    // Valid case: I₃ = 0.5, isospin = 0.5, charge = 2/3 (up quark)
    let result = WeakIsospin::new(0.5, 0.5, 2.0 / 3.0);
    assert!(result.is_ok(), "Valid parameters should succeed");

    let isospin = result.unwrap();
    assert_eq!(isospin.isospin, 0.5);
    assert_eq!(isospin.i3, 0.5);
    // hypercharge = 2 * (charge - i3) = 2 * (2/3 - 0.5) = 2 * 1/6 = 1/3
    assert!(
        (isospin.hypercharge - 1.0 / 3.0).abs() < 1e-10,
        "Hypercharge should be 1/3"
    );
}

#[test]
fn test_weak_isospin_new_boundary_case() {
    // Boundary case: I₃ = I exactly
    let result = WeakIsospin::new(0.5, 0.5, 0.0);
    assert!(result.is_ok(), "|I₃| = I should be valid");

    // Case: I₃ = -I exactly
    let result = WeakIsospin::new(0.5, -0.5, 0.0);
    assert!(result.is_ok(), "I₃ = -I should be valid");
}

#[test]
fn test_weak_isospin_left_coupling() {
    // g_L = g_V + g_A
    let lepton = WeakIsospin::lepton_doublet();
    let g_l = lepton.left_coupling();
    let g_v = lepton.vector_coupling();
    let g_a = lepton.axial_coupling();

    assert!(
        (g_l - (g_v + g_a)).abs() < 1e-10,
        "g_L = {} should equal g_V + g_A = {}",
        g_l,
        g_v + g_a
    );

    // For neutrino: g_L = 2 × I₃ = 1.0
    let nu = WeakIsospin::neutrino();
    let nu_g_l = nu.left_coupling();
    assert!(
        (nu_g_l - 1.0).abs() < 1e-10,
        "Neutrino g_L should be 1.0, got {}",
        nu_g_l
    );
}

#[test]
fn test_weak_isospin_right_coupling() {
    // g_R = g_V - g_A
    let lepton = WeakIsospin::lepton_doublet();
    let g_r = lepton.right_coupling();
    let g_v = lepton.vector_coupling();
    let g_a = lepton.axial_coupling();

    assert!(
        (g_r - (g_v - g_a)).abs() < 1e-10,
        "g_R = {} should equal g_V - g_A = {}",
        g_r,
        g_v - g_a
    );

    // For right-handed fermion: g_R = -2Q sin²θ_W (since I₃=0)
    let e_r = WeakIsospin::right_handed(-1.0);
    let e_r_g_r = e_r.right_coupling();
    let expected = 2.0 * SIN2_THETA_W; // -2 * (-1) * sin²θ_W = 2 sin²θ_W
    assert!(
        (e_r_g_r - expected).abs() < 1e-10,
        "Right-handed e g_R = {} should be {} ",
        e_r_g_r,
        expected
    );
}

#[test]
fn test_weak_isospin_default() {
    let default = WeakIsospin::default();
    let lepton = WeakIsospin::lepton_doublet();

    // Default should be lepton doublet
    assert_eq!(default.isospin, lepton.isospin);
    assert_eq!(default.i3, lepton.i3);
    assert_eq!(default.hypercharge, lepton.hypercharge);
}

#[test]
fn test_weak_field_ops_fermi_constant() {
    let weak = create_weak_field();
    let gf = weak.fermi_constant();

    assert!(
        (gf - FERMI_CONSTANT).abs() < 1e-15,
        "fermi_constant() should return FERMI_CONSTANT"
    );
}

#[test]
fn test_weak_field_ops_sin2_theta_w() {
    let weak = create_weak_field();
    let sin2 = weak.sin2_theta_w();

    assert!(
        (sin2 - SIN2_THETA_W).abs() < 1e-15,
        "sin2_theta_w() should return SIN2_THETA_W"
    );
}

#[test]
fn test_weak_field_strength() {
    let weak = create_weak_field();

    // weak_field_strength computes non-abelian field strength F_μν
    let f = weak.weak_field_strength();

    // Check shape: [num_points, dim, dim, lie_dim] = [1, 4, 4, 3]
    assert_eq!(f.shape(), &[1, 4, 4, 3]);

    // For zero connection, F = dA + [A, A] should be zero
    // Since A = 0, both terms are zero
    for val in f.as_slice() {
        assert!(
            val.abs() < 1e-10,
            "Field strength should be 0 for zero connection"
        );
    }
}

#[test]
fn test_neutral_current_propagator_nan_error() {
    let nu = WeakIsospin::neutrino();
    let result = WeakField::<f64>::neutral_current_propagator(f64::NAN, &nu);
    assert!(result.is_err(), "NaN momentum should return error");
}

#[test]
fn test_neutral_current_propagator_infinity_error() {
    let nu = WeakIsospin::neutrino();
    let result = WeakField::<f64>::neutral_current_propagator(f64::INFINITY, &nu);
    assert!(result.is_err(), "Infinite momentum should return error");
}

#[test]
fn test_weak_decay_width_nan_error() {
    let result = WeakField::<f64>::weak_decay_width(f64::NAN);
    assert!(result.is_err(), "NaN mass should return error");
}

#[test]
fn test_weak_decay_width_infinity_error() {
    let result = WeakField::<f64>::weak_decay_width(f64::INFINITY);
    assert!(result.is_err(), "Infinite mass should return error");
}

#[test]
fn test_right_handed_up_quark() {
    // Right-handed up quark: I=0, I₃=0, Q=+2/3
    let u_r = WeakIsospin::right_handed(2.0 / 3.0);
    assert_eq!(u_r.isospin, 0.0);
    assert_eq!(u_r.i3, 0.0);
    // Y = 2Q = 4/3
    assert!((u_r.hypercharge - 4.0 / 3.0).abs() < 1e-10);

    // Electric charge: Q = I₃ + Y/2 = 0 + (4/3)/2 = 2/3
    let q = u_r.electric_charge();
    assert!(
        (q - 2.0 / 3.0).abs() < 1e-10,
        "Right-handed u charge should be +2/3"
    );
}

#[test]
fn test_weak_isospin_edge_case_singlet() {
    // I = 0 singlet
    let singlet = WeakIsospin::new(0.0, 0.0, 0.0);
    assert!(singlet.is_ok(), "Singlet should be valid");

    let s = singlet.unwrap();
    assert_eq!(s.isospin, 0.0);
    assert_eq!(s.i3, 0.0);
    // Y = 2*(Q - I₃) = 0
    assert_eq!(s.hypercharge, 0.0);
}

#[test]
fn test_charged_current_propagator_high_energy() {
    // At high energy (q² >> M_W²), propagator → 1/q²
    let q2 = 1e6_f64; // Much larger than M_W² ≈ 6500
    let prop = WeakField::<f64>::charged_current_propagator(q2).unwrap();

    // Expected: 1/(q² - M_W²) ≈ 1/q² for large q²
    let expected = 1.0 / q2;
    assert!(
        (prop - expected).abs() / expected.abs() < 0.01,
        "High-energy propagator should approach 1/q²"
    );
}
