/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Comprehensive tests for Electroweak Theory — SU(2)_L × U(1)_Y unified gauge theory.
//!
//! Coverage includes:
//! - Physical constants and coupling relations
//! - Symmetry breaking (T-5: Higgs mechanism, VEV, potential minimum)
//! - Goldstone theorem verification (T-6: 3 eaten bosons)
//! - Gauge boson mixing (photon/Z extraction)
//! - Mass generation and ρ-parameter
//! - Cross-section calculations

use deep_causality_physics::theories::{ElectroweakField, ElectroweakOps, ElectroweakParams};
use deep_causality_physics::{ALPHA_EM, EM_COUPLING, HIGGS_MASS, TOP_MASS};
use deep_causality_tensor::CausalTensor;
use deep_causality_topology::{BaseTopology, Manifold, Simplex, SimplicialComplexBuilder};

// ============================================================================
// Test Helpers
// ============================================================================

fn create_electroweak_field() -> ElectroweakField<f64> {
    let mut builder = SimplicialComplexBuilder::new(0);
    let _ = builder.add_simplex(Simplex::new(vec![0]));
    let complex = builder.build().expect("Failed to build complex");
    let data = CausalTensor::new(vec![0.0], vec![1]).unwrap();
    let base = Manifold::new(complex, data, 0).expect("Failed to create manifold");

    // Electroweak: SU(2)×U(1), spacetime_dim=4, lie_algebra_dim=4
    let num_points = base.len();
    let conn = CausalTensor::zeros(&[num_points, 4, 4]);

    // Use constructor from ElectroweakOps specifically for convention compliance
    ElectroweakField::new_field(base, conn).expect("Failed to create ElectroweakField")
}

// ============================================================================
// Physical Constants Tests
// ============================================================================

#[test]
fn test_fine_structure_constant() {
    // α ≈ 1/137
    assert!((ALPHA_EM - 1.0 / 137.0).abs() < 1e-3, "α should be ≈ 1/137");
}

#[test]
fn test_em_coupling() {
    // e = √(4πα) ≈ 0.303
    let computed_e = (4.0 * std::f64::consts::PI * ALPHA_EM).sqrt();
    assert!(
        (EM_COUPLING - computed_e).abs() < 1e-3,
        "e = {} should satisfy e = √(4πα) = {}",
        EM_COUPLING,
        computed_e
    );
}

#[test]
fn test_higgs_mass() {
    // M_H ≈ 125 GeV
    const {
        assert!(HIGGS_MASS > 124.0 && HIGGS_MASS < 126.0);
    }
}

#[test]
fn test_top_mass() {
    // m_t ≈ 173 GeV
    const {
        assert!(TOP_MASS > 171.0 && TOP_MASS < 174.0);
    }
}

// ============================================================================
// Coupling Relation Tests
// ============================================================================

#[test]
fn test_coupling_relation_e_equals_g_sin() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // e = g sin θ_W
    let e = params.em_coupling();
    let g = params.g_coupling();
    let sin = params.sin_theta_w();

    assert!(
        (e - g * sin).abs() < 1e-6,
        "e = {} should equal g sin θ_W = {}",
        e,
        g * sin
    );
}

#[test]
fn test_coupling_relation_e_equals_gprime_cos() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // e = g' cos θ_W
    let e = params.em_coupling();
    let g_prime = params.g_prime_coupling();
    let cos = params.cos_theta_w();

    assert!(
        (e - g_prime * cos).abs() < 1e-6,
        "e = {} should equal g' cos θ_W = {}",
        e,
        g_prime * cos
    );
}

#[test]
fn test_tan_theta_relation() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // tan θ_W = g'/g
    let tan = params.tan_theta_w();
    let g = params.g_coupling();
    let g_prime = params.g_prime_coupling();

    assert!(
        (tan - g_prime / g).abs() < 1e-6,
        "tan θ_W should equal g'/g"
    );
}

// ============================================================================
// Mass Generation Tests
// ============================================================================

#[test]
fn test_w_mass_computed() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // M_W = g v / 2
    let m_w = params.w_mass_computed();
    let g = params.g_coupling();
    let v = params.higgs_vev();
    let expected = g * v / 2.0;

    assert!(
        (m_w - expected).abs() < 1.0,
        "M_W = {} should equal gv/2 = {}",
        m_w,
        expected
    );
}

#[test]
fn test_z_mass_relation() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // M_Z = M_W / cos θ_W
    let m_w = params.w_mass_computed();
    let m_z = params.z_mass_computed();
    let cos = params.cos_theta_w();

    assert!(
        (m_z - m_w / cos).abs() < 1e-6,
        "M_Z should equal M_W / cos θ_W"
    );
}

#[test]
fn test_fermion_mass() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // m_f = y_f v / √2
    let y_top = params.top_yukawa();
    let v = params.higgs_vev();
    let m_top = params.fermion_mass(y_top);
    let expected = y_top * v / 2.0_f64.sqrt();

    assert!(
        (m_top - expected).abs() < 1e-6,
        "m_t should equal y_t v / √2"
    );
}

// ============================================================================
// Symmetry Breaking Tests (T-5)
// ============================================================================

#[test]
fn test_higgs_potential_minimum() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // V(φ) has minimum at |φ| = v/√2
    let v_over_sqrt2 = params.higgs_vev() / 2.0_f64.sqrt();

    // Potential at minimum should be negative
    let v_min = params.higgs_potential(v_over_sqrt2);
    assert!(v_min < 0.0, "V(v/√2) = {} should be negative", v_min);

    // Potential at zero should be zero
    let v_zero = params.higgs_potential(0.0);
    assert!(v_zero.abs() < 1e-10, "V(0) should be 0");
}

#[test]
fn test_symmetry_breaking_verified() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // VEV should satisfy potential minimum condition
    assert!(
        params.symmetry_breaking_verified(),
        "VEV should satisfy ∂V/∂|φ| = 0"
    );
}

#[test]
fn test_higgs_quartic() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // λ = M_H² / (2v²)
    let lambda = params.higgs_quartic();
    let expected = HIGGS_MASS * HIGGS_MASS / (2.0 * params.higgs_vev().powi(2));

    assert!(
        (lambda - expected).abs() < 1e-6,
        "λ = {} should equal M_H²/(2v²) = {}",
        lambda,
        expected
    );
}

// ============================================================================
// Goldstone Theorem Tests (T-6)
// ============================================================================

#[test]
fn test_goldstone_count() {
    // SU(2)×U(1) → U(1)_EM: 4 - 1 = 3 Goldstones
    assert_eq!(
        ElectroweakParams::<f64>::goldstone_count(),
        3,
        "3 Goldstones eaten by W⁺, W⁻, Z"
    );
}

#[test]
fn test_gauge_boson_masses() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    let (m_w, m_z, m_a) = params.gauge_boson_masses();

    // W and Z massive, photon massless
    assert!(m_w > 70.0 && m_w < 90.0, "M_W should be ~80 GeV");
    assert!(m_z > 80.0 && m_z < 100.0, "M_Z should be ~91 GeV");
    assert!(m_a.abs() < 1e-10, "Photon should be massless");
}

// ============================================================================
// Rho Parameter Tests
// ============================================================================

#[test]
fn test_rho_parameter() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // At tree level with exact masses, ρ = 1
    // With experimental PDG masses, small deviation expected: ρ ≈ 1.01
    let rho = params.rho_parameter();
    assert!(
        (rho - 1.0).abs() < 0.02,
        "ρ = {} should be ≈ 1 at tree level (within radiative corrections)",
        rho
    );
}

#[test]
fn test_rho_deviation() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // With PDG masses, deviation is ~1% (radiative corrections)
    let deviation = params.rho_deviation();
    assert!(
        deviation < 0.02,
        "ρ deviation = {} should be < 2%",
        deviation
    );
}

// ============================================================================
// Gauge Boson Mixing Tests
// ============================================================================

#[test]
fn test_extract_photon() {
    let ew = create_electroweak_field();

    let photon = ew.extract_photon();
    assert!(photon.is_ok(), "Photon extraction should succeed");

    let qed = photon.unwrap();
    assert!(
        qed.gauge_group_name().contains("U(1)"),
        "Photon should be U(1) field"
    );
    assert!(qed.is_abelian(), "Photon field should be abelian");
}

#[test]
fn test_extract_z() {
    let ew = create_electroweak_field();

    let z = ew.extract_z();
    assert!(z.is_ok(), "Z extraction should succeed");

    let z_field = z.unwrap();
    assert!(
        z_field.gauge_group_name().contains("U(1)"),
        "Z should be U(1) neutral current"
    );
}

// ============================================================================
// Cross-Section Tests
// ============================================================================

#[test]
fn test_neutrino_electron_cross_section() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // At 10 GeV
    let sigma = params.neutrino_electron_cross_section(10.0);
    assert!(sigma.is_ok(), "Cross section should compute");
    assert!(sigma.unwrap() > 0.0, "Cross section should be positive");
}

#[test]
fn test_neutrino_electron_cross_section_invalid() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    let result = params.neutrino_electron_cross_section(-1.0);
    assert!(result.is_err(), "Negative energy should return error");
}

#[test]
fn test_z_resonance_cross_section() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // At Z pole
    let m_z = params.z_mass();
    let gamma_z = 2.5; // approximate width

    let sigma = params.z_resonance_cross_section(m_z, gamma_z);
    assert!(sigma.is_ok(), "Resonance cross section should compute");

    // At resonance, σ(M_Z) should be at maximum
    let sigma_at_pole = sigma.unwrap();
    let sigma_off_pole = params.z_resonance_cross_section(50.0, gamma_z).unwrap();

    assert!(
        sigma_at_pole > sigma_off_pole,
        "σ at pole should be larger than off-pole"
    );
}

// ============================================================================
// Field Structure Tests
// ============================================================================

#[test]
fn test_electroweak_field_is_non_abelian() {
    let ew = create_electroweak_field();
    assert!(!ew.is_abelian(), "Electroweak should be non-abelian");
}

#[test]
fn test_electroweak_field_west_coast() {
    let ew = create_electroweak_field();
    assert!(
        ew.is_west_coast(),
        "Electroweak should use West Coast signature"
    );
}

// ============================================================================
// ElectroweakParams Coverage Gap Tests
// ============================================================================

#[test]
fn test_with_mixing_angle_zero_error() {
    let result = ElectroweakParams::<f64>::with_mixing_angle(0.0);
    assert!(result.is_err(), "sin²θ_W = 0 should return error");
}

#[test]
fn test_with_mixing_angle_negative_error() {
    let result = ElectroweakParams::<f64>::with_mixing_angle(-0.1);
    assert!(result.is_err(), "sin²θ_W < 0 should return error");
}

#[test]
fn test_with_mixing_angle_one_error() {
    let result = ElectroweakParams::<f64>::with_mixing_angle(1.0);
    assert!(result.is_err(), "sin²θ_W = 1 should return error");
}

#[test]
fn test_with_mixing_angle_greater_than_one_error() {
    let result = ElectroweakParams::<f64>::with_mixing_angle(1.5);
    assert!(result.is_err(), "sin²θ_W > 1 should return error");
}

#[test]
fn test_with_mixing_angle_valid() {
    let result = ElectroweakParams::<f64>::with_mixing_angle(0.23);
    assert!(result.is_ok(), "Valid sin²θ_W should succeed");
    let params = result.unwrap();
    assert!((params.sin2_theta_w() - 0.23).abs() < 1e-10);
}

#[test]
fn test_rho_parameter_computed() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // At tree level, ρ_computed should be exactly 1.0 by construction
    let rho = params.rho_parameter_computed();
    assert!(
        (rho - 1.0).abs() < 1e-6,
        "ρ_computed = {} should be 1.0 by construction",
        rho
    );
}

#[test]
fn test_rho_effective() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();

    // Without radiative corrections, ρ_eff = 1.0
    assert_eq!(
        params.rho_effective(),
        1.0,
        "ρ_eff should be 1.0 without corrections"
    );

    // With precision mode, ρ_eff = 1 + Δρ > 1
    let precision_params = ElectroweakParams::<f64>::standard_model_precision();
    let rho_eff = precision_params.rho_effective();
    assert!(
        rho_eff > 1.0,
        "ρ_eff = {} should be > 1 with radiative corrections",
        rho_eff
    );
}

#[test]
fn test_corrections_accessor() {
    // Standard model without precision has no corrections
    let params = ElectroweakParams::<f64>::standard_model();
    assert!(
        params.corrections().is_none(),
        "Standard model should have no corrections"
    );

    // Precision mode has corrections
    let precision_params = ElectroweakParams::<f64>::standard_model_precision();
    let corrections = precision_params.corrections();
    assert!(
        corrections.is_some(),
        "Precision mode should have corrections"
    );

    let c = corrections.unwrap();
    assert!(c.delta_rho > 0.0, "Δρ should be positive");
}

#[test]
fn test_z_resonance_cross_section_negative_energy_error() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();
    let result = params.z_resonance_cross_section(-10.0, 2.5);
    assert!(result.is_err(), "Negative energy should return error");
}

#[test]
fn test_z_resonance_cross_section_zero_energy_error() {
    let params: ElectroweakParams<f64> = ElectroweakParams::standard_model();
    let result = params.z_resonance_cross_section(0.0, 2.5);
    assert!(result.is_err(), "Zero energy should return error");
}

#[test]
fn test_standard_model_precision_masses() {
    let params = ElectroweakParams::<f64>::standard_model_precision();

    // W mass should be physical (80.3 - 80.4 GeV)
    let m_w = params.w_mass_computed();
    assert!(
        m_w > 80.0 && m_w < 81.0,
        "M_W = {} should be in physical range",
        m_w
    );

    // Z mass should be fixed at PDG value
    let m_z = params.z_mass_computed();
    assert!(
        (m_z - 91.1876).abs() < 0.01,
        "M_Z = {} should be ~91.19 GeV",
        m_z
    );
}

#[test]
fn test_z_partial_width_fermion_coverage() {
    let params = ElectroweakParams::<f64>::standard_model_precision();

    // Test quark channel (with color factor and QCD corrections)
    let gamma_u = params.z_partial_width_fermion(true, 0.5, 2.0 / 3.0);
    assert!(gamma_u > 0.0, "Up quark width should be positive");

    // Test lepton channel
    let gamma_e = params.z_partial_width_fermion(false, -0.5, -1.0);
    assert!(gamma_e > 0.0, "Electron width should be positive");

    // Quarks should have larger width due to color factor
    assert!(
        gamma_u > gamma_e,
        "Quark width {} should be > lepton width {}",
        gamma_u,
        gamma_e
    );
}

#[test]
fn test_z_total_width_computed_physical() {
    let params = ElectroweakParams::<f64>::standard_model_precision();
    let gamma_z = params.z_total_width_computed();

    // PDG value: Γ_Z ≈ 2.4952 GeV
    assert!(
        gamma_z > 2.4 && gamma_z < 2.6,
        "Γ_Z = {} should be ~2.5 GeV",
        gamma_z
    );
}

#[test]
fn test_z_hadronic_width_computed() {
    let params = ElectroweakParams::<f64>::standard_model_precision();
    let gamma_had = params.z_hadronic_width_computed();

    // Hadronic width is the dominant contribution (~1.7 GeV)
    assert!(
        gamma_had > 1.5 && gamma_had < 2.0,
        "Γ_had = {} should be ~1.7 GeV",
        gamma_had
    );
}
