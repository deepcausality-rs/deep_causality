/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

const TOLERANCE: f64 = 1e-12;
use deep_causality_physics::{CentralBody, EARTH_GM, EARTH_J2, EARTH_RADIUS_EQUATORIAL};

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
