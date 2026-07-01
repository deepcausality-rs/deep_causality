/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Forward relativistic clock kernels — validated against the textbook GPS relativistic split (Ashby 2003)
//! and the Gap-3 FS-3 study numbers.

use deep_causality_physics::{
    EARTH_GM, EARTH_RADIUS_EQUATORIAL, relativistic_clock_drift_rate_kernel,
    relativistic_clock_offset_kernel,
};

const SECONDS_PER_DAY: f64 = 86_400.0;
const GPS_RADIUS_M: f64 = 26_560_000.0;

#[test]
fn gps_relativistic_split_matches_textbook() {
    // GPS satellite clock vs a geoid clock: +45.7 (grav) − 7.2 (vel) = +38.5 µs/day (Ashby 2003).
    let v_gps = (EARTH_GM / GPS_RADIUS_M).sqrt();

    // Gravitational-only: a clock at GPS radius vs the geoid, both at rest.
    let grav =
        relativistic_clock_offset_kernel(GPS_RADIUS_M, 0.0, EARTH_RADIUS_EQUATORIAL, 0.0, EARTH_GM)
            .unwrap();
    let grav_us_day = grav * SECONDS_PER_DAY * 1.0e6;
    assert!(
        (grav_us_day - 45.7).abs() < 1.0,
        "gravitational offset {grav_us_day} µs/day should be ≈ +45.7"
    );

    // Velocity-only: the GPS speed vs a clock at rest at the SAME radius (isolate the kinematic term).
    let vel =
        relativistic_clock_offset_kernel(GPS_RADIUS_M, v_gps, GPS_RADIUS_M, 0.0, EARTH_GM).unwrap();
    let vel_us_day = vel * SECONDS_PER_DAY * 1.0e6;
    assert!(
        (vel_us_day + 7.2).abs() < 0.5,
        "velocity offset {vel_us_day} µs/day should be ≈ −7.2"
    );

    // Net: GPS clock (moving, high) vs geoid clock (at rest).
    let net = relativistic_clock_offset_kernel(
        GPS_RADIUS_M,
        v_gps,
        EARTH_RADIUS_EQUATORIAL,
        0.0,
        EARTH_GM,
    )
    .unwrap();
    let net_us_day = net * SECONDS_PER_DAY * 1.0e6;
    assert!(
        (net_us_day - 38.5).abs() < 1.0,
        "net offset {net_us_day} µs/day should be ≈ +38.5"
    );
}

#[test]
fn drift_rate_signs_and_magnitude() {
    // A clock deeper in the well and/or faster runs slow ⇒ negative drift relative to coordinate time.
    let surface =
        relativistic_clock_drift_rate_kernel(EARTH_RADIUS_EQUATORIAL, 0.0, EARTH_GM).unwrap();
    let gps = relativistic_clock_drift_rate_kernel(GPS_RADIUS_M, 0.0, EARTH_GM).unwrap();
    assert!(surface < 0.0, "surface clock drift < 0 (deep in the well)");
    assert!(gps > surface, "higher clock runs faster (less negative)");
    // Magnitude is the tiny weak-field ratio ~1e-10.
    assert!(surface.abs() < 1e-8 && surface.abs() > 1e-11);
}

#[test]
fn offset_equals_difference_of_drift_rates() {
    let v = 3000.0;
    let r_a = 8.0e6;
    let r_b = EARTH_RADIUS_EQUATORIAL;
    let offset = relativistic_clock_offset_kernel(r_a, v, r_b, 0.0, EARTH_GM).unwrap();
    let diff = relativistic_clock_drift_rate_kernel(r_a, v, EARTH_GM).unwrap()
        - relativistic_clock_drift_rate_kernel(r_b, 0.0, EARTH_GM).unwrap();
    assert!(
        (offset - diff).abs() < 1e-18,
        "offset must equal the drift-rate difference"
    );
}

#[test]
fn reentry_blackout_carry_is_tens_of_metres() {
    // FS-3 reentry demo: 180 s GNSS-denied window at ~7.65 km/s, 71 km altitude, vs the surface.
    let r = EARTH_RADIUS_EQUATORIAL + 71_000.0;
    let rate = relativistic_clock_offset_kernel(r, 7650.0, EARTH_RADIUS_EQUATORIAL, 0.0, EARTH_GM)
        .unwrap();
    let offset_ns = rate * 180.0 * 1.0e9;
    let ranging_m = offset_ns.abs() * 0.299_792_458;
    assert!(
        (10.0..200.0).contains(&offset_ns.abs()),
        "blackout offset {offset_ns} ns should be tens of ns"
    );
    assert!(
        ranging_m > 1.0,
        "ranging drift {ranging_m} m should be metres-scale"
    );
}

#[test]
fn rejects_bad_inputs() {
    assert!(relativistic_clock_drift_rate_kernel(0.0, 0.0, EARTH_GM).is_err());
    assert!(relativistic_clock_drift_rate_kernel(-1.0, 0.0, EARTH_GM).is_err());
    assert!(relativistic_clock_drift_rate_kernel(7.0e6, -1.0, EARTH_GM).is_err());
    assert!(relativistic_clock_drift_rate_kernel(7.0e6, 0.0, 0.0).is_err());
    assert!(relativistic_clock_drift_rate_kernel(7.0e6, 0.0, -1.0).is_err());
    // The offset kernel propagates either clock's error.
    assert!(relativistic_clock_offset_kernel(-1.0, 0.0, 7.0e6, 0.0, EARTH_GM).is_err());
    assert!(relativistic_clock_offset_kernel(7.0e6, 0.0, 0.0, 0.0, EARTH_GM).is_err());
}
