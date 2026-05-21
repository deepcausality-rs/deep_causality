/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the [`solve_gm_analytical`] causal wrapper.
//!
//! The wrapper lifts the kernel's `Result<R, PhysicsError>` into a
//! `PropagatingEffect<R>`. These tests verify both the success branch
//! (kernel result becomes `EffectValue::Value(gm)`) and the error branch
//! (kernel error becomes a `PropagatingEffect::from_error` variant).

use deep_causality_core::EffectValue;
use deep_causality_physics::{
    CentralBody, EARTH_GM, EARTH_RADIUS_EQUATORIAL, SPEED_OF_LIGHT, SpaceTimeCoordinate,
    solve_gm_analytical,
};

const RELATIVE_TOLERANCE: f64 = 1e-8;

// =============================================================================
// Helpers (mirror solve_gm_tests so each test file is self-contained)
// =============================================================================

fn forward_drift_rate(target_gm: f64, r: f64, v: f64, z: f64, body: &CentralBody<f64>) -> f64 {
    let cos_theta = z / r;
    let legendre_p2 = 0.5 * (3.0 * cos_theta * cos_theta - 1.0);
    let r_cubed = r * r * r;
    let req_sq = body.equatorial_radius_m * body.equatorial_radius_m;
    let inv_r_eff = 1.0 / r - body.j2 * req_sq * legendre_p2 / r_cubed;
    let phi = -target_gm * inv_r_eff;
    let c_sq = SPEED_OF_LIGHT * SPEED_OF_LIGHT;
    phi / c_sq - 0.5 * v * v / c_sq
}

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

// =============================================================================
// Success path
// =============================================================================

#[test]
fn test_wrapper_success_returns_value() {
    let body = CentralBody::<f64>::new(EARTH_GM, EARTH_RADIUS_EQUATORIAL, 0.0);
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

    let effect = solve_gm_analytical(&coord_a, &coord_b, &body);
    assert!(
        effect.error.is_none(),
        "expected no error, got {:?}",
        effect.error
    );
    match effect.value {
        EffectValue::Value(gm) => {
            let rel = (gm - EARTH_GM).abs() / EARTH_GM;
            assert!(
                rel < RELATIVE_TOLERANCE,
                "wrapper recovery rel={} exceeds tolerance {}",
                rel,
                RELATIVE_TOLERANCE
            );
        }
        other => panic!("expected EffectValue::Value, got {:?}", other),
    }
}

#[test]
fn test_wrapper_success_with_j2() {
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

    let effect = solve_gm_analytical(&coord_a, &coord_b, &body);
    assert!(effect.error.is_none());
    match effect.value {
        EffectValue::Value(gm) => {
            let rel = (gm - EARTH_GM).abs() / EARTH_GM;
            assert!(rel < RELATIVE_TOLERANCE);
        }
        other => panic!("expected EffectValue::Value, got {:?}", other),
    }
}

// =============================================================================
// Error path — propagation through CausalityError::from(PhysicsError)
// =============================================================================

#[test]
fn test_wrapper_error_on_zero_radius() {
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

    let effect = solve_gm_analytical(&coord_a, &coord_b, &body);
    assert!(
        effect.error.is_some(),
        "expected error, got effect={:?}",
        effect
    );
    // Value should be the default for the f64 success type — which is 0.0.
    match effect.value {
        EffectValue::None => {}
        other => panic!("expected EffectValue::None on error, got {:?}", other),
    }
}

#[test]
fn test_wrapper_error_on_negative_radius() {
    let body = CentralBody::EARTH_JGM3;
    let coord_a = SpaceTimeCoordinate::<f64> {
        timestamp: 0,
        sat_id: 0,
        r_m: -1.0e6,
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

    let effect = solve_gm_analytical(&coord_a, &coord_b, &body);
    assert!(effect.error.is_some());
}

#[test]
fn test_wrapper_error_on_insufficient_separation() {
    let body = CentralBody::<f64>::new(EARTH_GM, EARTH_RADIUS_EQUATORIAL, 0.0);
    // Identical coords → epsilon guard fires.
    let coord = build_coord(
        EARTH_GM,
        2.93e7,
        3650.0,
        [2.93e7, 0.0, 0.0],
        [0.0, 3650.0, 0.0],
        &body,
    );
    let effect = solve_gm_analytical(&coord, &coord, &body);
    assert!(
        effect.error.is_some(),
        "expected error, got effect={:?}",
        effect
    );
}

// =============================================================================
// Round-trip composability — wrapper output is usable downstream
// =============================================================================

#[test]
fn test_wrapper_value_can_be_extracted() {
    let body = CentralBody::<f64>::new(EARTH_GM, EARTH_RADIUS_EQUATORIAL, 0.0);
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

    let effect = solve_gm_analytical(&coord_a, &coord_b, &body);
    let extracted: Option<f64> = match effect.value {
        EffectValue::Value(gm) => Some(gm),
        _ => None,
    };
    assert!(extracted.is_some());
    let gm = extracted.unwrap();
    assert!(gm > 0.0);
    let rel = (gm - EARTH_GM).abs() / EARTH_GM;
    assert!(rel < RELATIVE_TOLERANCE);
}

#[test]
fn test_wrapper_logs_field_present() {
    // PropagatingEffect carries a logs field — verify it's at least
    // accessible (default-constructed) on a successful result.
    let body = CentralBody::<f64>::new(EARTH_GM, EARTH_RADIUS_EQUATORIAL, 0.0);
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

    let effect = solve_gm_analytical(&coord_a, &coord_b, &body);
    // Just confirm the effect is well-formed (no panics, fields accessible).
    let _ = format!("{:?}", effect.logs);
}
