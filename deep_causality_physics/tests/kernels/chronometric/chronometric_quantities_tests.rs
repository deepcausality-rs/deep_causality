/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the chronometric quantity types: `CentralBody` and `SpaceTimeCoordinate`.

use deep_causality_physics::{
    CentralBody, EARTH_GM, EARTH_J2, EARTH_RADIUS_EQUATORIAL, SPEED_OF_LIGHT, SpaceTimeCoordinate,
};

const TOLERANCE: f64 = 1e-12;

// =============================================================================
// CentralBody — construction
// =============================================================================

#[test]
fn test_central_body_new_basic() {
    let body = CentralBody::<f64>::new(1.0e14, 6.0e6, 1.0e-3);
    assert!((body.gm - 1.0e14).abs() < TOLERANCE);
    assert!((body.equatorial_radius_m - 6.0e6).abs() < TOLERANCE);
    assert!((body.j2 - 1.0e-3).abs() < TOLERANCE);
}

#[test]
fn test_central_body_new_zero_j2() {
    // J2 = 0 corresponds to a perfectly spherical body — used for forward
    // modelling tests where the J2 correction is intentionally disabled.
    let body = CentralBody::<f64>::new(EARTH_GM, EARTH_RADIUS_EQUATORIAL, 0.0);
    assert_eq!(body.j2, 0.0);
}

#[test]
fn test_central_body_new_mars_like() {
    // Mars: GM ≈ 4.28e13, R_eq ≈ 3.396e6, J2 ≈ 1.96e-3
    let mars = CentralBody::<f64>::new(4.28e13, 3.396e6, 1.96e-3);
    assert!((mars.gm - 4.28e13).abs() < TOLERANCE);
    assert!((mars.equatorial_radius_m - 3.396e6).abs() < TOLERANCE);
    assert!((mars.j2 - 1.96e-3).abs() < TOLERANCE);
}

// =============================================================================
// CentralBody — EARTH_JGM3 constant
// =============================================================================

#[test]
fn test_central_body_earth_jgm3_gm() {
    let body = CentralBody::EARTH_JGM3;
    assert_eq!(body.gm, EARTH_GM);
}

#[test]
fn test_central_body_earth_jgm3_radius() {
    let body = CentralBody::EARTH_JGM3;
    assert_eq!(body.equatorial_radius_m, EARTH_RADIUS_EQUATORIAL);
}

#[test]
fn test_central_body_earth_jgm3_j2() {
    let body = CentralBody::EARTH_JGM3;
    assert_eq!(body.j2, EARTH_J2);
}

#[test]
fn test_central_body_earth_jgm3_consistency() {
    // The published EARTH_GM, EARTH_RADIUS_EQUATORIAL, and EARTH_J2 values must
    // all be present and reasonable. This guards against accidental constant
    // drift in the source tree.
    let body = CentralBody::EARTH_JGM3;
    assert!(body.gm > 3.9e14 && body.gm < 4.0e14);
    assert!(body.equatorial_radius_m > 6.3e6 && body.equatorial_radius_m < 6.4e6);
    assert!(body.j2 > 1.0e-3 && body.j2 < 2.0e-3);
}

// =============================================================================
// CentralBody — derive macros
// =============================================================================

#[test]
fn test_central_body_clone() {
    let original = CentralBody::EARTH_JGM3;
    let cloned = Clone::clone(&original);
    assert_eq!(original, cloned);
}

#[test]
fn test_central_body_copy() {
    let original = CentralBody::EARTH_JGM3;
    let copied = original; // Copy semantics
    assert_eq!(original, copied);
}

#[test]
fn test_central_body_debug() {
    let body = CentralBody::EARTH_JGM3;
    let debug_str = format!("{:?}", body);
    assert!(debug_str.contains("CentralBody"));
    assert!(debug_str.contains("gm"));
    assert!(debug_str.contains("j2"));
}

#[test]
fn test_central_body_partial_eq_equal() {
    let a = CentralBody::<f64>::new(1.0e14, 6.0e6, 1.0e-3);
    let b = CentralBody::<f64>::new(1.0e14, 6.0e6, 1.0e-3);
    assert_eq!(a, b);
}

#[test]
fn test_central_body_partial_eq_different_gm() {
    let a = CentralBody::<f64>::new(1.0e14, 6.0e6, 1.0e-3);
    let b = CentralBody::<f64>::new(2.0e14, 6.0e6, 1.0e-3);
    assert_ne!(a, b);
}

#[test]
fn test_central_body_partial_eq_different_radius() {
    let a = CentralBody::<f64>::new(1.0e14, 6.0e6, 1.0e-3);
    let b = CentralBody::<f64>::new(1.0e14, 7.0e6, 1.0e-3);
    assert_ne!(a, b);
}

#[test]
fn test_central_body_partial_eq_different_j2() {
    let a = CentralBody::<f64>::new(1.0e14, 6.0e6, 1.0e-3);
    let b = CentralBody::<f64>::new(1.0e14, 6.0e6, 2.0e-3);
    assert_ne!(a, b);
}

// =============================================================================
// SpaceTimeCoordinate — construction and field access
// =============================================================================

#[test]
fn test_space_time_coordinate_basic_construction() {
    let coord = SpaceTimeCoordinate::<f64> {
        timestamp: 1_700_000_000,
        sat_id: 14,
        r_m: 2.93e7,
        v_ms: 3650.0,
        clock_bias_s: -1.234e-7,
        position: [1.5e7, 2.1e7, 1.2e7],
        velocity: [-2400.0, 1800.0, 1000.0],
        clock_drift_rate: -3.5e-10,
    };
    assert_eq!(coord.timestamp, 1_700_000_000);
    assert_eq!(coord.sat_id, 14);
    assert!((coord.r_m - 2.93e7).abs() < TOLERANCE);
    assert!((coord.v_ms - 3650.0).abs() < TOLERANCE);
    assert!((coord.clock_bias_s - (-1.234e-7)).abs() < TOLERANCE);
    assert_eq!(coord.position, [1.5e7, 2.1e7, 1.2e7]);
    assert_eq!(coord.velocity, [-2400.0, 1800.0, 1000.0]);
    assert!((coord.clock_drift_rate - (-3.5e-10)).abs() < TOLERANCE);
}

#[test]
fn test_space_time_coordinate_clone() {
    let coord = sample_coord();
    let cloned = Clone::clone(&coord);
    assert_eq!(coord, cloned);
}

#[test]
fn test_space_time_coordinate_copy() {
    let coord = sample_coord();
    let copied = coord; // Copy semantics
    assert_eq!(coord, copied);
}

#[test]
fn test_space_time_coordinate_debug() {
    let coord = sample_coord();
    let debug_str = format!("{:?}", coord);
    assert!(debug_str.contains("SpaceTimeCoordinate"));
    assert!(debug_str.contains("timestamp"));
    assert!(debug_str.contains("sat_id"));
    assert!(debug_str.contains("r_m"));
}

#[test]
fn test_space_time_coordinate_partial_eq_equal() {
    let a = sample_coord();
    let b = sample_coord();
    assert_eq!(a, b);
}

#[test]
fn test_space_time_coordinate_partial_eq_different_timestamp() {
    let mut a = sample_coord();
    let b = sample_coord();
    a.timestamp = 999;
    assert_ne!(a, b);
}

#[test]
fn test_space_time_coordinate_partial_eq_different_sat_id() {
    let mut a = sample_coord();
    let b = sample_coord();
    a.sat_id = 99;
    assert_ne!(a, b);
}

// =============================================================================
// SpaceTimeCoordinate — get_total_bias
// =============================================================================

#[test]
fn test_get_total_bias_zero_radial_velocity() {
    // For a perfectly circular orbit, r ⊥ v, so r·v = 0.
    // The relativistic correction term -2(r·v)/c² vanishes, and
    // get_total_bias returns clock_bias_s unchanged.
    let coord = SpaceTimeCoordinate::<f64> {
        timestamp: 0,
        sat_id: 1,
        r_m: 2.93e7,
        v_ms: 3650.0,
        clock_bias_s: -1.0e-7,
        // r·v = 1·0 + 0·1 + 0·0 = 0 (perpendicular vectors)
        position: [2.93e7, 0.0, 0.0],
        velocity: [0.0, 3650.0, 0.0],
        clock_drift_rate: 0.0,
    };
    let total = coord.get_total_bias();
    assert!((total - coord.clock_bias_s).abs() < 1e-15);
}

#[test]
fn test_get_total_bias_positive_radial_velocity() {
    // r·v > 0 (outward radial motion) → correction is negative,
    // so total_bias < clock_bias_s.
    let coord = SpaceTimeCoordinate::<f64> {
        timestamp: 0,
        sat_id: 1,
        r_m: 2.93e7,
        v_ms: 3650.0,
        clock_bias_s: 0.0,
        // r·v = 2.93e7 * 1000 = 2.93e10 (positive)
        position: [2.93e7, 0.0, 0.0],
        velocity: [1000.0, 3500.0, 0.0],
        clock_drift_rate: 0.0,
    };
    let total = coord.get_total_bias();
    let expected = -2.0 * (2.93e7 * 1000.0) / (SPEED_OF_LIGHT * SPEED_OF_LIGHT);
    assert!((total - expected).abs() < 1e-20);
    assert!(total < 0.0);
}

#[test]
fn test_get_total_bias_negative_radial_velocity() {
    // r·v < 0 (inward radial motion) → correction is positive,
    // so total_bias > clock_bias_s.
    let coord = SpaceTimeCoordinate::<f64> {
        timestamp: 0,
        sat_id: 1,
        r_m: 2.93e7,
        v_ms: 3650.0,
        clock_bias_s: 0.0,
        position: [2.93e7, 0.0, 0.0],
        velocity: [-1000.0, 3500.0, 0.0],
        clock_drift_rate: 0.0,
    };
    let total = coord.get_total_bias();
    let expected = -2.0 * (2.93e7 * -1000.0) / (SPEED_OF_LIGHT * SPEED_OF_LIGHT);
    assert!((total - expected).abs() < 1e-20);
    assert!(total > 0.0);
}

#[test]
fn test_get_total_bias_magnitude_realistic() {
    // For an eccentric Galileo satellite (E14-like, r·v on order 10^10),
    // the periodic correction has amplitude ~few hundred ns. Verify the
    // magnitude is in that range, not orders of magnitude off.
    let coord = SpaceTimeCoordinate::<f64> {
        timestamp: 0,
        sat_id: 14,
        r_m: 2.93e7,
        v_ms: 3650.0,
        clock_bias_s: 0.0,
        // dot_rv ≈ 1.7e10 m²/s (typical for eccentric MEO at maximum)
        position: [2.0e7, 1.5e7, 1.0e7],
        velocity: [2000.0, -1500.0, -1000.0],
        clock_drift_rate: 0.0,
    };
    let dot_rv = 2.0e7 * 2000.0 + 1.5e7 * -1500.0 + 1.0e7 * -1000.0;
    let expected = -2.0 * dot_rv / (SPEED_OF_LIGHT * SPEED_OF_LIGHT);
    let total = coord.get_total_bias();
    assert!((total - expected).abs() < 1e-20);
    // Magnitude should be in the nanosecond range (|total| < 1µs).
    assert!(total.abs() < 1e-6);
}

#[test]
fn test_get_total_bias_preserves_existing_bias() {
    // get_total_bias adds the correction onto clock_bias_s; verify the
    // base bias is preserved when r·v = 0.
    let coord = SpaceTimeCoordinate::<f64> {
        timestamp: 0,
        sat_id: 1,
        r_m: 2.93e7,
        v_ms: 3650.0,
        clock_bias_s: 5.5e-7,
        position: [2.93e7, 0.0, 0.0],
        velocity: [0.0, 3650.0, 0.0],
        clock_drift_rate: 0.0,
    };
    let total = coord.get_total_bias();
    assert!((total - 5.5e-7).abs() < 1e-15);
}

// =============================================================================
// Test helpers
// =============================================================================

fn sample_coord() -> SpaceTimeCoordinate<f64> {
    SpaceTimeCoordinate::<f64> {
        timestamp: 1_700_000_000,
        sat_id: 14,
        r_m: 2.93e7,
        v_ms: 3650.0,
        clock_bias_s: -1.234e-7,
        position: [1.5e7, 2.1e7, 1.2e7],
        velocity: [-2400.0, 1800.0, 1000.0],
        clock_drift_rate: -3.5e-10,
    }
}
