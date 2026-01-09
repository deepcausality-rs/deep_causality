/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for metrics.rs - Spacetime metric constructors and error handling

use deep_causality_physics::theories::general_relativity::{
    flrw_metric_at, kerr_metric_at, minkowski_metric, schwarzschild_christoffel_at,
    schwarzschild_kretschmann, schwarzschild_metric_at,
};
use std::f64::consts::PI;

// ============================================================================
// Metric Constructors
// ============================================================================

#[test]
fn test_minkowski_spacetime() {
    let metric = minkowski_metric();
    assert_eq!(metric.shape(), &[4, 4]);

    // Verify signature (-+++)
    let diag = metric.as_slice();
    assert_eq!(diag[0], -1.0); // g_00
    assert_eq!(diag[5], 1.0); // g_11
    assert_eq!(diag[10], 1.0); // g_22
    assert_eq!(diag[15], 1.0); // g_33
}

#[test]
fn test_schwarzschild_metric_properties() {
    let mass = 1.0;
    let r = 10.0;
    let metric = schwarzschild_metric_at(mass, r).expect("Should create metric");

    let data = metric.as_slice();
    // g_tt = -(1 - 2M/r) = -(1 - 2/10) = -0.8
    assert!((data[0] - (-0.8)).abs() < 1e-10);

    // g_rr = 1/(1 - 2M/r) = 1/0.8 = 1.25
    assert!((data[5] - 1.25).abs() < 1e-10);

    // g_theta = r^2 = 100
    assert!((data[10] - 100.0).abs() < 1e-10);
}

#[test]
fn test_schwarzschild_curvature_invariants() {
    let mass = 1.0;
    let r = 10.0;

    // Direct calculation of K
    let k_exact = schwarzschild_kretschmann(mass, r);
    // K = 48 M^2 / r^6 = 48 * 1 / 10^6 = 0.000048
    assert!((k_exact - 48.0e-6).abs() < 1e-12);
}

#[test]
fn test_kerr_black_hole() {
    let mass = 2.0;
    let r = 20.0;
    let theta = PI / 2.0; // Equatorial

    // Case 1: Non-rotating (a=0) limit -> Should match Schwarzschild
    let kerr0 = kerr_metric_at(mass, 0.0, r, theta).unwrap();
    let schw = schwarzschild_metric_at(mass, r).unwrap();

    let k_data = kerr0.as_slice();
    let s_data = schw.as_slice();

    for i in 0..16 {
        assert!(
            (k_data[i] - s_data[i]).abs() < 1e-10,
            "Mismatch at index {}",
            i
        );
    }

    // Case 2: Extreme Kerr (a=M)
    // At horizon r = M + sqrt(M^2 - a^2) = M
    // Here we test outside horizon
    let _kerr_rot = kerr_metric_at(mass, mass * 0.9, r, theta).unwrap();
}

#[test]
fn test_flrw_cosmology() {
    let a = 2.0; // Scale factor
    let r = 5.0;
    let theta = PI / 2.0;

    // Flat universe (k=0)
    let flrw = flrw_metric_at(a, 0.0, r, theta).unwrap();
    let data = flrw.as_slice();

    // ds^2 = -dt^2 + a^2(dr^2 + r^2...)
    assert_eq!(data[0], -1.0); // g_tt
    assert_eq!(data[5], a * a); // g_rr for flat k=0
}

// ============================================================================
// Schwarzschild Christoffel Components
// ============================================================================

#[test]
fn test_schwarzschild_christoffel_components() {
    let mass = 1.0;
    let r = 10.0;
    let r_s = 2.0 * mass;
    let f = 1.0 - r_s / r; // 0.8
    let f_prime = r_s / (r * r); // 0.02

    let christoffel = schwarzschild_christoffel_at(mass, r).expect("Should create Christoffel");
    let gamma = christoffel.as_slice();

    // Verify shape [4, 4, 4] = 64 elements
    assert_eq!(christoffel.shape(), &[4, 4, 4]);
    assert_eq!(gamma.len(), 64);

    // Helper to compute index Γ^a_bc in flattened [4,4,4] tensor
    let idx = |a: usize, b: usize, c: usize| a * 16 + b * 4 + c;

    // Γ^t_tr = Γ^t_rt = f'/(2f) = 0.02 / 1.6 = 0.0125
    let g_t_tr = f_prime / (2.0 * f);
    assert!(
        (gamma[idx(0, 1, 0)] - g_t_tr).abs() < 1e-10,
        "Γ^t_rt = {} (expected {})",
        gamma[idx(0, 1, 0)],
        g_t_tr
    );

    // Γ^r_tt = f * f' / 2 = 0.8 * 0.02 / 2 = 0.008
    let g_r_tt = f * f_prime / 2.0;
    assert!(
        (gamma[idx(1, 0, 0)] - g_r_tt).abs() < 1e-10,
        "Γ^r_tt = {} (expected {})",
        gamma[idx(1, 0, 0)],
        g_r_tt
    );

    // Γ^r_rr = -f' / (2f) = -0.0125
    let g_r_rr = -f_prime / (2.0 * f);
    assert!(
        (gamma[idx(1, 1, 1)] - g_r_rr).abs() < 1e-10,
        "Γ^r_rr mismatch"
    );

    // Γ^θ_rθ = 1/r = 0.1
    assert!(
        (gamma[idx(2, 1, 2)] - 0.1).abs() < 1e-10,
        "Γ^θ_rθ = {} (expected 0.1)",
        gamma[idx(2, 1, 2)]
    );

    // Γ^φ_rφ = 1/r = 0.1
    assert!((gamma[idx(3, 1, 3)] - 0.1).abs() < 1e-10, "Γ^φ_rφ mismatch");
}

#[test]
fn test_schwarzschild_kretschmann_various_values() {
    // K = 48 M² / r⁶
    let test_cases = [
        (1.0, 10.0, 48.0 / 1_000_000.0),
        (2.0, 10.0, 48.0 * 4.0 / 1_000_000.0),
        (1.0, 100.0, 48.0 / 1e12),
    ];

    for (mass, r, expected) in test_cases {
        let k = schwarzschild_kretschmann(mass, r);
        assert!(
            (k - expected).abs() / expected.max(1e-15) < 1e-10,
            "K({}, {}) = {} (expected {})",
            mass,
            r,
            k,
            expected
        );
    }
}

// ============================================================================
// Error Paths
// ============================================================================

#[test]
fn test_kerr_metric_negative_radius_error() {
    let result = kerr_metric_at(1.0, 0.5, -5.0, PI / 2.0);
    assert!(result.is_err(), "Negative radius should return error");
}

#[test]
fn test_kerr_metric_horizon_singularity() {
    let mass = 1.0;
    let a = 0.0; // Non-rotating

    // At r = 2M (horizon), Δ = r² - 2Mr + a² = 4 - 4 = 0
    // This should trigger singularity error
    let result = kerr_metric_at(mass, a, 2.0 * mass, PI / 2.0);
    assert!(result.is_err(), "Horizon (Δ=0) should return error");
}

#[test]
fn test_kerr_metric_ring_singularity() {
    // Ring singularity: Σ = r² + a²cos²θ → 0
    // This happens at r=0, θ=π/2 with a≠0
    let result = kerr_metric_at(1.0, 0.5, 0.0, PI / 2.0);
    // Actually at r=0, Σ=0 + a²×0 = 0 for θ=π/2
    assert!(
        result.is_err(),
        "Ring singularity (Σ=0) should return error"
    );
}

#[test]
fn test_flrw_metric_nonpositive_scale_factor() {
    let result = flrw_metric_at(0.0, 0.0, 5.0, PI / 2.0);
    assert!(result.is_err(), "Zero scale factor should return error");

    let result = flrw_metric_at(-1.0, 0.0, 5.0, PI / 2.0);
    assert!(result.is_err(), "Negative scale factor should return error");
}

#[test]
fn test_flrw_metric_coordinate_singularity() {
    // Singularity at 1 - kr² = 0 → r = 1/√k for k > 0
    let k = 1.0;
    let r = 1.0; // At this point, 1 - kr² = 0

    let result = flrw_metric_at(1.0, k, r, PI / 2.0);
    assert!(
        result.is_err(),
        "Coordinate singularity (1-kr²=0) should return error"
    );
}

#[test]
fn test_schwarzschild_metric_nonpositive_radius() {
    let result = schwarzschild_metric_at(1.0, 0.0);
    assert!(result.is_err(), "Zero radius should return error");

    let result = schwarzschild_metric_at(1.0, -5.0);
    assert!(result.is_err(), "Negative radius should return error");
}

#[test]
fn test_schwarzschild_metric_inside_horizon() {
    let mass = 1.0;
    let r_s = 2.0 * mass; // Schwarzschild radius

    // At horizon
    let result = schwarzschild_metric_at(mass, r_s);
    assert!(result.is_err(), "At horizon should return error");

    // Inside horizon
    let result = schwarzschild_metric_at(mass, r_s * 0.5);
    assert!(result.is_err(), "Inside horizon should return error");
}

#[test]
fn test_schwarzschild_christoffel_error_paths() {
    // Zero radius
    let result = schwarzschild_christoffel_at(1.0, 0.0);
    assert!(result.is_err(), "Zero radius should return error");

    // Negative radius
    let result = schwarzschild_christoffel_at(1.0, -5.0);
    assert!(result.is_err(), "Negative radius should return error");

    // Inside horizon
    let result = schwarzschild_christoffel_at(1.0, 1.5); // r_s = 2
    assert!(result.is_err(), "Inside horizon should return error");
}
