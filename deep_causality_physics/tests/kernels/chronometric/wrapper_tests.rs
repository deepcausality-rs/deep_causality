/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Tests for the [`solve_gm_analytical`] causal wrapper.
//!
//! The wrapper lifts the kernel's `Result<R, PhysicsError>` into a
//! `PropagatingEffect<R>`. These tests verify both the success branch
//! (kernel result becomes `gm`) and the error branch
//! (kernel error becomes a `PropagatingEffect::from_error` variant).

use deep_causality_physics::utils_tests::chronometric_build_coord as build_coord;
use deep_causality_physics::{
    CentralBody, EARTH_GM, EARTH_RADIUS_EQUATORIAL, SpaceTimeCoordinate, solve_gm_analytical,
};

const RELATIVE_TOLERANCE: f64 = 1e-8;

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
        effect.error().is_none(),
        "expected no error, got {:?}",
        effect.error()
    );
    match effect.value() {
        Some(gm) => {
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
    assert!(effect.error().is_none());
    match effect.value() {
        Some(gm) => {
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

    // The refusal must name its cause. `solve_gm_tests.rs` pins the kernel's variant
    // (TopologyError); the wrapper forwards its text through a `CausalityError`.
    let effect = solve_gm_analytical(&coord_a, &coord_b, &body);
    assert!(
        effect
            .error()
            .expect("a zero radius must be refused")
            .to_string()
            .contains("Non-positive radial distance"),
        "unexpected refusal: {:?}",
        effect.error()
    );
    // Value and error are one channel: an errored effect provably carries no value.
    assert!(effect.value().is_none(), "errored effect carries no value");
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
    // A wrapper forwards its kernel's refusal through a `CausalityError`, which keeps the
    // `PhysicsError` text. Asserting the text is how the *reason* stays pinned once the
    // variant itself is erased by the effect channel.
    let err = effect.error().expect("the call must fail");
    assert!(
        err.to_string().contains("Topology Error"),
        "expected a Topology Error refusal, got {err}"
    );
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
    // Identical coordinates leave no radial separation to invert, and the refusal must say so
    // rather than merely being a refusal.
    let effect = solve_gm_analytical(&coord, &coord, &body);
    assert!(
        effect
            .error()
            .expect("identical coordinates must be refused")
            .to_string()
            .contains("Insufficient"),
        "unexpected refusal: {:?}",
        effect.error()
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
    let extracted: Option<f64> = effect.value().copied();
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

    // A successful effect carries a value and no error, and its log channel is present and
    // separate from that value. Formatting the logs and discarding the string, as this test used
    // to, asserted nothing at all.
    let effect = solve_gm_analytical(&coord_a, &coord_b, &body);
    assert!(effect.error().is_none(), "the call succeeds");
    assert!(
        effect.value().is_some(),
        "a successful effect carries a value"
    );

    // `EffectLog` compares by message sequence, so an untouched channel equals a fresh one.
    assert_eq!(
        *effect.logs(),
        deep_causality_core::EffectLog::new(),
        "this kernel emits no log entries, so the channel must stay empty"
    );
}
