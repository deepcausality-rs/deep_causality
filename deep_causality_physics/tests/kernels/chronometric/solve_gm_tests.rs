/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for [`solve_gm_analytical_kernel`].
//!
//! Strategy: forward-model two `SpaceTimeCoordinate` samples whose `clock_drift_rate`
//! values are constructed from a chosen target `GM`, then verify the kernel inverts
//! them back to that target. This is a round-trip identity test — the only way an
//! analytical inversion can be verified without external truth data.

use deep_causality_physics::{
    CentralBody, EARTH_GM, PhysicsErrorEnum, SPEED_OF_LIGHT, SpaceTimeCoordinate,
    solve_gm_analytical_kernel,
};

/// Inverse-relative tolerance for the round-trip GM recovery.
/// Float64 arithmetic across the inversion accumulates ~1e-10 relative error
/// from the differences of nearly-equal numbers in the redshift / kinetic
/// terms; a tolerance of 1e-8 leaves ample headroom for environmental noise.
const RELATIVE_TOLERANCE: f64 = 1e-8;

// =============================================================================
// Forward-model helper
// =============================================================================

/// Compute the clock drift rate that solves the 1PN clock equation
/// $\dot\tau = 1 + \Phi(r,\theta)/c^2 - v^2/(2c^2)$
/// for a given target `gm`, J2-corrected effective potential.
fn forward_drift_rate(target_gm: f64, r: f64, v: f64, z: f64, body: &CentralBody<f64>) -> f64 {
    let inv_r_eff = inv_r_effective(r, z, body);
    let phi = -target_gm * inv_r_eff_to_potential_factor(inv_r_eff, r);
    // Φ for the J2-corrected geopotential: Φ = -GM × (1/r_eff_geometric)
    // where 1/r_eff_geometric = 1/r - J2·R_eq² P2(cos θ)/r³
    let _ = phi; // kept for clarity; the actual formula below
    let phi = -target_gm * inv_r_eff;
    let c_sq = SPEED_OF_LIGHT * SPEED_OF_LIGHT;
    phi / c_sq - 0.5 * v * v / c_sq
}

/// 1/r_eff = 1/r − J2 · R_eq² · P₂(cos θ) / r³ where P₂ = (3cos²θ − 1)/2.
fn inv_r_effective(r: f64, z: f64, body: &CentralBody<f64>) -> f64 {
    let cos_theta = z / r;
    let legendre_p2 = 0.5 * (3.0 * cos_theta * cos_theta - 1.0);
    let r_cubed = r * r * r;
    let req_sq = body.equatorial_radius_m * body.equatorial_radius_m;
    1.0 / r - body.j2 * req_sq * legendre_p2 / r_cubed
}

#[inline]
fn inv_r_eff_to_potential_factor(inv_r_eff: f64, _r: f64) -> f64 {
    inv_r_eff
}

/// Build a SpaceTimeCoordinate with a forward-modeled clock_drift_rate
/// consistent with the supplied target_gm and body parameters.
fn build_coord(
    target_gm: f64,
    r: f64,
    v: f64,
    position: [f64; 3],
    velocity: [f64; 3],
    body: &CentralBody<f64>,
) -> SpaceTimeCoordinate<f64> {
    SpaceTimeCoordinate::<f64> {
        timestamp: 0,
        sat_id: 0,
        r_m: r,
        v_ms: v,
        clock_bias_s: 0.0,
        position,
        velocity,
        clock_drift_rate: forward_drift_rate(target_gm, r, v, position[2], body),
    }
}

fn assert_relative(actual: f64, expected: f64, tol: f64) {
    let rel_err = (actual - expected).abs() / expected.abs();
    assert!(
        rel_err < tol,
        "relative error {} exceeds tolerance {} (actual={}, expected={})",
        rel_err,
        tol,
        actual,
        expected
    );
}

// =============================================================================
// Round-trip recovery — spherical body (J2 = 0)
// =============================================================================

#[test]
fn test_round_trip_recovery_spherical_earth_equatorial() {
    let body = CentralBody::<f64>::new(EARTH_GM, 6.378e6, 0.0); // J2 = 0
    let coord_a = build_coord(
        EARTH_GM,
        2.93e7,
        3650.0,
        [2.93e7, 0.0, 0.0],
        [0.0, 3650.0, 0.0],
        &body,
    );
    let coord_b = build_coord(
        EARTH_GM,
        2.95e7,
        3640.0,
        [2.95e7, 0.0, 0.0],
        [0.0, 3640.0, 0.0],
        &body,
    );

    let recovered = solve_gm_analytical_kernel(&coord_a, &coord_b, &body).unwrap();
    assert_relative(recovered, EARTH_GM, RELATIVE_TOLERANCE);
}

#[test]
fn test_round_trip_recovery_spherical_earth_polar() {
    // Polar coordinates (z != 0). Without J2 the result is unaffected by colatitude,
    // so the round trip should still recover GM cleanly.
    let body = CentralBody::<f64>::new(EARTH_GM, 6.378e6, 0.0);
    let coord_a = build_coord(
        EARTH_GM,
        2.93e7,
        3650.0,
        [0.0, 0.0, 2.93e7],
        [0.0, 3650.0, 0.0],
        &body,
    );
    let coord_b = build_coord(
        EARTH_GM,
        2.95e7,
        3640.0,
        [0.0, 0.0, 2.95e7],
        [0.0, 3640.0, 0.0],
        &body,
    );

    let recovered = solve_gm_analytical_kernel(&coord_a, &coord_b, &body).unwrap();
    assert_relative(recovered, EARTH_GM, RELATIVE_TOLERANCE);
}

#[test]
fn test_round_trip_recovery_spherical_mars_like() {
    // Body-agnostic property of the kernel: with consistent forward modelling,
    // any GM in the weak-field regime is recovered to the same precision.
    let target_gm = 4.28e13; // Mars-like
    let body = CentralBody::<f64>::new(target_gm, 3.396e6, 0.0);
    let coord_a = build_coord(
        target_gm,
        6.0e6,
        3500.0,
        [6.0e6, 0.0, 0.0],
        [0.0, 3500.0, 0.0],
        &body,
    );
    let coord_b = build_coord(
        target_gm,
        7.0e6,
        3300.0,
        [7.0e6, 0.0, 0.0],
        [0.0, 3300.0, 0.0],
        &body,
    );

    let recovered = solve_gm_analytical_kernel(&coord_a, &coord_b, &body).unwrap();
    assert_relative(recovered, target_gm, RELATIVE_TOLERANCE);
}

// =============================================================================
// Round-trip recovery — with J2 oblateness
// =============================================================================

#[test]
fn test_round_trip_recovery_with_j2_equatorial() {
    let body = CentralBody::EARTH_JGM3;
    // Equatorial pair (z = 0): P₂(cos θ) = -1/2 maximum J2 effect.
    let coord_a = build_coord(
        EARTH_GM,
        2.93e7,
        3650.0,
        [2.93e7, 0.0, 0.0],
        [0.0, 3650.0, 0.0],
        &body,
    );
    let coord_b = build_coord(
        EARTH_GM,
        2.95e7,
        3640.0,
        [2.95e7, 0.0, 0.0],
        [0.0, 3640.0, 0.0],
        &body,
    );

    let recovered = solve_gm_analytical_kernel(&coord_a, &coord_b, &body).unwrap();
    assert_relative(recovered, EARTH_GM, RELATIVE_TOLERANCE);
}

#[test]
fn test_round_trip_recovery_with_j2_polar() {
    let body = CentralBody::EARTH_JGM3;
    // Polar pair (z = r): P₂(cos θ) = +1 maximum opposite-sign J2 effect.
    let coord_a = build_coord(
        EARTH_GM,
        2.93e7,
        3650.0,
        [0.0, 0.0, 2.93e7],
        [0.0, 3650.0, 0.0],
        &body,
    );
    let coord_b = build_coord(
        EARTH_GM,
        2.95e7,
        3640.0,
        [0.0, 0.0, 2.95e7],
        [0.0, 3640.0, 0.0],
        &body,
    );

    let recovered = solve_gm_analytical_kernel(&coord_a, &coord_b, &body).unwrap();
    assert_relative(recovered, EARTH_GM, RELATIVE_TOLERANCE);
}

#[test]
fn test_round_trip_recovery_with_j2_mid_latitude() {
    let body = CentralBody::EARTH_JGM3;
    // Mid-latitude pair near the magic angle θ ≈ 54.7° where 3cos²θ = 1.
    let r = 2.93e7;
    let z = r * (1.0_f64 / 3.0_f64).sqrt(); // cos θ = 1/√3
    let coord_a = build_coord(
        EARTH_GM,
        r,
        3650.0,
        [r * 0.5, 0.0, z],
        [0.0, 3650.0, 0.0],
        &body,
    );
    let r_b = 2.95e7;
    let z_b = r_b * (1.0_f64 / 3.0_f64).sqrt();
    let coord_b = build_coord(
        EARTH_GM,
        r_b,
        3640.0,
        [r_b * 0.5, 0.0, z_b],
        [0.0, 3640.0, 0.0],
        &body,
    );

    let recovered = solve_gm_analytical_kernel(&coord_a, &coord_b, &body).unwrap();
    assert_relative(recovered, EARTH_GM, RELATIVE_TOLERANCE);
}

// =============================================================================
// J2 sensitivity: same coords, different body J2 → different recovered GM
// =============================================================================

#[test]
fn test_j2_sensitivity_changes_result() {
    // Forward-model with J2 = EARTH_J2, but invert with J2 = 0 — the recovered
    // GM should differ from EARTH_GM, demonstrating that the J2 term matters.
    let body_with_j2 = CentralBody::EARTH_JGM3;
    let body_no_j2 = CentralBody::<f64>::new(EARTH_GM, EARTH_RADIUS_EQUATORIAL, 0.0);

    let coord_a = build_coord(
        EARTH_GM,
        2.93e7,
        3650.0,
        [2.93e7, 0.0, 0.0],
        [0.0, 3650.0, 0.0],
        &body_with_j2,
    );
    let coord_b = build_coord(
        EARTH_GM,
        2.95e7,
        3640.0,
        [2.95e7, 0.0, 0.0],
        [0.0, 3640.0, 0.0],
        &body_with_j2,
    );

    // Inverting with the wrong (J2=0) body gives a measurably wrong GM.
    let recovered_wrong = solve_gm_analytical_kernel(&coord_a, &coord_b, &body_no_j2).unwrap();
    let recovered_right = solve_gm_analytical_kernel(&coord_a, &coord_b, &body_with_j2).unwrap();

    // Right inversion is at round-trip precision; wrong one is biased.
    assert_relative(recovered_right, EARTH_GM, RELATIVE_TOLERANCE);
    let bias_relative = (recovered_wrong - EARTH_GM).abs() / EARTH_GM;
    assert!(
        bias_relative > 1e-9,
        "expected J2-omission to introduce a measurable bias; got {} relative",
        bias_relative
    );
}

// We need EARTH_RADIUS_EQUATORIAL in scope for the test above.
use deep_causality_physics::EARTH_RADIUS_EQUATORIAL;

// =============================================================================
// Error: non-positive radius
// =============================================================================

#[test]
fn test_error_zero_radius_on_first_coord() {
    let body = CentralBody::EARTH_JGM3;
    let coord_a = SpaceTimeCoordinate::<f64> {
        timestamp: 0,
        sat_id: 0,
        r_m: 0.0,
        v_ms: 3650.0,
        clock_bias_s: 0.0,
        position: [0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0],
        clock_drift_rate: 0.0,
    };
    let coord_b = build_coord(
        EARTH_GM,
        2.95e7,
        3640.0,
        [2.95e7, 0.0, 0.0],
        [0.0, 3640.0, 0.0],
        &body,
    );
    let result = solve_gm_analytical_kernel(&coord_a, &coord_b, &body);
    assert!(result.is_err());
    match result.unwrap_err().0 {
        PhysicsErrorEnum::TopologyError(msg) => {
            assert!(msg.contains("Non-positive radial distance"));
        }
        e => panic!("expected TopologyError, got {:?}", e),
    }
}

#[test]
fn test_error_negative_radius_on_first_coord() {
    let body = CentralBody::EARTH_JGM3;
    let coord_a = SpaceTimeCoordinate::<f64> {
        timestamp: 0,
        sat_id: 0,
        r_m: -1.0e6, // negative radius
        v_ms: 3650.0,
        clock_bias_s: 0.0,
        position: [-1.0e6, 0.0, 0.0],
        velocity: [0.0, 3650.0, 0.0],
        clock_drift_rate: 0.0,
    };
    let coord_b = build_coord(
        EARTH_GM,
        2.95e7,
        3640.0,
        [2.95e7, 0.0, 0.0],
        [0.0, 3640.0, 0.0],
        &body,
    );
    let result = solve_gm_analytical_kernel(&coord_a, &coord_b, &body);
    assert!(result.is_err());
    match result.unwrap_err().0 {
        PhysicsErrorEnum::TopologyError(msg) => {
            assert!(msg.contains("Non-positive radial distance"));
        }
        e => panic!("expected TopologyError, got {:?}", e),
    }
}

#[test]
fn test_error_zero_radius_on_second_coord() {
    let body = CentralBody::EARTH_JGM3;
    let coord_a = build_coord(
        EARTH_GM,
        2.93e7,
        3650.0,
        [2.93e7, 0.0, 0.0],
        [0.0, 3650.0, 0.0],
        &body,
    );
    let coord_b = SpaceTimeCoordinate::<f64> {
        timestamp: 0,
        sat_id: 0,
        r_m: 0.0,
        v_ms: 3650.0,
        clock_bias_s: 0.0,
        position: [0.0, 0.0, 0.0],
        velocity: [0.0, 0.0, 0.0],
        clock_drift_rate: 0.0,
    };
    let result = solve_gm_analytical_kernel(&coord_a, &coord_b, &body);
    assert!(result.is_err());
    match result.unwrap_err().0 {
        PhysicsErrorEnum::TopologyError(msg) => {
            assert!(msg.contains("Non-positive radial distance"));
        }
        e => panic!("expected TopologyError, got {:?}", e),
    }
}

// =============================================================================
// Error: insufficient effective radial separation (epsilon guard)
// =============================================================================

#[test]
fn test_error_insufficient_radial_separation() {
    let body = CentralBody::<f64>::new(EARTH_GM, EARTH_RADIUS_EQUATORIAL, 0.0);
    // Two coords at *identical* radius and same colatitude — the denominator
    // (1/r_eff_a − 1/r_eff_b) vanishes and the epsilon guard fires.
    let coord_a = build_coord(
        EARTH_GM,
        2.93e7,
        3650.0,
        [2.93e7, 0.0, 0.0],
        [0.0, 3650.0, 0.0],
        &body,
    );
    let coord_b = build_coord(
        EARTH_GM,
        2.93e7,
        3650.0,
        [2.93e7, 0.0, 0.0],
        [0.0, 3650.0, 0.0],
        &body,
    );

    let result = solve_gm_analytical_kernel(&coord_a, &coord_b, &body);
    assert!(result.is_err());
    match result.unwrap_err().0 {
        PhysicsErrorEnum::TopologyError(msg) => {
            assert!(msg.contains("Insufficient"));
        }
        e => panic!("expected TopologyError, got {:?}", e),
    }
}

// =============================================================================
// Sign convention check
// =============================================================================

#[test]
fn test_higher_altitude_clock_drifts_higher() {
    // Forward-modelled drift_b at higher altitude (b) should be MORE positive
    // (less-negative) than drift_a deep in the well. This confirms the kernel's
    // sign convention: clock_drift_rate ≡ dτ/dt − 1, negative deeper in the well.
    let body = CentralBody::<f64>::new(EARTH_GM, EARTH_RADIUS_EQUATORIAL, 0.0);
    let coord_a = build_coord(
        EARTH_GM,
        2.0e7,
        4000.0,
        [2.0e7, 0.0, 0.0],
        [0.0, 4000.0, 0.0],
        &body,
    );
    let coord_b = build_coord(
        EARTH_GM,
        4.0e7,
        3000.0,
        [4.0e7, 0.0, 0.0],
        [0.0, 3000.0, 0.0],
        &body,
    );

    assert!(coord_b.clock_drift_rate > coord_a.clock_drift_rate);
    // And the round-trip still recovers GM cleanly.
    let recovered = solve_gm_analytical_kernel(&coord_a, &coord_b, &body).unwrap();
    assert_relative(recovered, EARTH_GM, RELATIVE_TOLERANCE);
}

// =============================================================================
// Magnitude sanity: kernel returns a positive value of expected scale
// =============================================================================

#[test]
fn test_recovered_gm_positive_and_correct_scale() {
    let body = CentralBody::EARTH_JGM3;
    let coord_a = build_coord(
        EARTH_GM,
        2.93e7,
        3650.0,
        [2.93e7, 0.0, 0.0],
        [0.0, 3650.0, 0.0],
        &body,
    );
    let coord_b = build_coord(
        EARTH_GM,
        2.95e7,
        3640.0,
        [2.95e7, 0.0, 0.0],
        [0.0, 3640.0, 0.0],
        &body,
    );
    let gm = solve_gm_analytical_kernel(&coord_a, &coord_b, &body).unwrap();
    assert!(gm > 0.0);
    // Within an order of magnitude of EARTH_GM.
    assert!(gm > 1.0e14);
    assert!(gm < 1.0e15);
}
