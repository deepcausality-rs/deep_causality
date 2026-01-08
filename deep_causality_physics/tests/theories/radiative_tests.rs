/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Comprehensive tests for Electroweak Radiative Corrections.
//!
//! Coverage includes:
//! - Veltman screening correction (Δρ)
//! - Weak radiative correction (Δr_weak)
//! - Effective weak mixing angle
//! - Full W mass solver with radiative corrections

use deep_causality_physics::theories::electroweak::{
    calculate_delta_r_weak, calculate_delta_rho, calculate_effective_angle, solve_w_mass,
};
use deep_causality_physics::{ALPHA_EM, ALPHA_EM_MZ, FERMI_CONSTANT, TOP_MASS, Z_MASS};

// ============================================================================
// Veltman Screening Correction (Δρ) Tests
// ============================================================================

#[test]
fn test_calculate_delta_rho_with_top_mass() {
    // Δρ = (3 · G_F · m_t²) / (8 · π² · √2)
    let delta_rho: f64 = calculate_delta_rho(TOP_MASS);

    // Δρ should be positive (top mass enhances ρ)
    assert!(
        delta_rho > 0.0,
        "Δρ should be positive for non-zero top mass"
    );

    // Expected value ≈ 0.009 - 0.010 for m_t ≈ 173 GeV
    assert!(
        delta_rho > 0.008 && delta_rho < 0.012,
        "Δρ = {} should be ≈ 0.01 for top mass ≈ 173 GeV",
        delta_rho
    );
}

#[test]
fn test_calculate_delta_rho_zero_mass() {
    // Δρ should be 0 for zero top mass
    let delta_rho: f64 = calculate_delta_rho(0.0);
    assert!(
        delta_rho.abs() < 1e-15,
        "Δρ should be 0 for zero top mass, got {}",
        delta_rho
    );
}

#[test]
fn test_calculate_delta_rho_scales_with_mass_squared() {
    // Δρ ∝ m_t²
    let delta_rho_100: f64 = calculate_delta_rho(100.0);
    let delta_rho_200: f64 = calculate_delta_rho(200.0);

    // If m doubles, Δρ should quadruple
    let ratio = delta_rho_200 / delta_rho_100;
    assert!(
        (ratio - 4.0).abs() < 1e-10,
        "Δρ should scale as m², ratio = {}",
        ratio
    );
}

// ============================================================================
// Weak Radiative Correction (Δr_weak) Tests
// ============================================================================

#[test]
fn test_calculate_delta_r_weak() {
    // Δr_weak = - (cos²θ_W / sin²θ_W) · Δρ
    let sin2_theta_w = 0.23122; // PDG value
    let delta_rho = 0.01;

    let delta_r_weak: f64 = calculate_delta_r_weak(sin2_theta_w, delta_rho);

    // cot²θ = cos²θ/sin²θ = (1 - 0.231)/0.231 ≈ 3.33
    // Δr_weak ≈ -3.33 × 0.01 ≈ -0.033
    assert!(
        delta_r_weak < 0.0,
        "Δr_weak should be negative, got {}",
        delta_r_weak
    );
    assert!(
        (delta_r_weak + 0.033).abs() < 0.01,
        "Δr_weak = {} should be ≈ -0.033",
        delta_r_weak
    );
}

#[test]
fn test_calculate_delta_r_weak_zero_delta_rho() {
    let sin2_theta_w = 0.23122;
    let delta_r_weak: f64 = calculate_delta_r_weak(sin2_theta_w, 0.0);

    assert!(
        delta_r_weak.abs() < 1e-15,
        "Δr_weak should be 0 when Δρ = 0"
    );
}

// ============================================================================
// Effective Weak Mixing Angle Tests
// ============================================================================

#[test]
fn test_calculate_effective_angle() {
    // sin²θ_eff = sin²θ_W + cos²θ_W · Δρ
    let sin2_theta_w = 0.23122;
    let delta_rho = 0.01;

    let sin2_eff: f64 = calculate_effective_angle(sin2_theta_w, delta_rho);

    // cos²θ = 1 - 0.231 = 0.769
    // sin²θ_eff = 0.231 + 0.769 × 0.01 ≈ 0.2387
    let expected = sin2_theta_w + (1.0 - sin2_theta_w) * delta_rho;
    assert!(
        (sin2_eff - expected).abs() < 1e-10,
        "sin²θ_eff = {} should equal {}",
        sin2_eff,
        expected
    );
}

#[test]
fn test_calculate_effective_angle_zero_delta_rho() {
    let sin2_theta_w = 0.23122;
    let sin2_eff: f64 = calculate_effective_angle(sin2_theta_w, 0.0);

    assert!(
        (sin2_eff - sin2_theta_w).abs() < 1e-15,
        "sin²θ_eff should equal sin²θ_W when Δρ = 0"
    );
}

#[test]
fn test_effective_angle_increases_with_delta_rho() {
    let sin2_theta_w = 0.23122;
    let eff_1: f64 = calculate_effective_angle(sin2_theta_w, 0.005);
    let eff_2: f64 = calculate_effective_angle(sin2_theta_w, 0.010);

    assert!(eff_2 > eff_1, "Larger Δρ should give larger sin²θ_eff");
}

// ============================================================================
// W Mass Solver Tests
// ============================================================================

#[test]
fn test_solve_w_mass_physical_inputs() {
    let mz = Z_MASS;
    let top_mass = TOP_MASS;
    let alpha_mz = ALPHA_EM_MZ;
    let alpha_0 = ALPHA_EM;
    let g_f = FERMI_CONSTANT;

    let result = solve_w_mass(mz, top_mass, alpha_mz, alpha_0, g_f);
    assert!(
        result.is_ok(),
        "solve_w_mass should succeed with physical inputs"
    );

    let corrections = result.unwrap();

    // W mass should be in physical range (80.3 - 80.4 GeV)
    assert!(
        corrections.w_mass_corrected > 80.0 && corrections.w_mass_corrected < 81.0,
        "M_W = {} should be ≈ 80.4 GeV",
        corrections.w_mass_corrected
    );

    // Δρ should be positive
    assert!(corrections.delta_rho > 0.0, "Δρ should be positive");

    // Δr should be positive (dominated by QED running in standard Δr)
    // Note: Δr_std ≈ 1 - (α(0)/α(M_Z)) × (1 - Δr_weak) ≈ 0.035-0.04
    assert!(
        corrections.delta_r > 0.0,
        "Standard Δr should be positive, got {}",
        corrections.delta_r
    );

    // Effective sin²θ should be close to on-shell value
    assert!(
        corrections.sin2_theta_eff > 0.22 && corrections.sin2_theta_eff < 0.24,
        "sin²θ_eff = {} should be ≈ 0.231",
        corrections.sin2_theta_eff
    );
}

#[test]
fn test_solve_w_mass_negative_discriminant() {
    // Create conditions that would cause negative discriminant
    // This happens when A is too large relative to M_Z²
    // Using very small G_F to make A very large
    let mz = Z_MASS;
    let top_mass = TOP_MASS;
    let alpha_mz = ALPHA_EM_MZ;
    let alpha_0 = ALPHA_EM;
    let tiny_gf = 1e-20; // Unrealistically small G_F

    let result = solve_w_mass(mz, top_mass, alpha_mz, alpha_0, tiny_gf);
    assert!(
        result.is_err(),
        "solve_w_mass should fail with tiny G_F causing negative discriminant"
    );
}

#[test]
fn test_solve_w_mass_zero_top_mass() {
    // Zero top mass means Δρ = 0
    let result = solve_w_mass(Z_MASS, 0.0, ALPHA_EM_MZ, ALPHA_EM, FERMI_CONSTANT);
    assert!(
        result.is_ok(),
        "solve_w_mass should succeed with zero top mass"
    );

    let corrections = result.unwrap();
    assert!(
        corrections.delta_rho.abs() < 1e-15,
        "Δρ should be 0 for zero top mass"
    );
}

// ============================================================================
// Integration with ElectroweakParams
// ============================================================================

#[test]
fn test_radiative_corrections_through_electroweak_params() {
    use deep_causality_physics::theories::ElectroweakParams;

    let params = ElectroweakParams::<f64>::standard_model_precision();

    // Verify corrections are computed
    let corrections = params.corrections();
    assert!(
        corrections.is_some(),
        "Precision mode should have corrections"
    );

    let c = corrections.unwrap();
    assert!(c.delta_rho > 0.0, "Δρ should be positive");
    assert!(c.w_mass_corrected > 80.0, "M_W should be > 80 GeV");
}
