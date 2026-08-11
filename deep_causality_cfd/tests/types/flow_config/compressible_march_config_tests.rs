/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The compressible march config: the builder's required sections, the seed conversion, and the
//! descent schedule's table validation and interpolation.

use deep_causality_cfd::{
    AtmosphereRow, CfdConfigBuilder, DescentSchedule, MarchStop, PlumeImprint, QttObserve,
};
use deep_causality_tensor::Truncation;

fn rows() -> Vec<AtmosphereRow<f64>> {
    vec![
        AtmosphereRow {
            altitude_m: 30_000.0,
            n_tot: 8.0e23,
            temperature: 226.0,
            sound_speed: 301.0,
        },
        AtmosphereRow {
            altitude_m: 61_000.0,
            n_tot: 1.3e21,
            temperature: 250.0,
            sound_speed: 317.0,
        },
        AtmosphereRow {
            altitude_m: 90_000.0,
            n_tot: 7.0e19,
            temperature: 187.0,
            sound_speed: 274.0,
        },
    ]
}

/// The corridor's dimensional anchors as the builder takes them: `(t_ref, n_ref, u_ref)`.
const REFERENCE: (f64, f64, f64) = (8_044.0, 2.645e22, 376.0);

#[test]
fn schedule_rejects_short_or_unsorted_tables() {
    assert!(DescentSchedule::new(rows()[..1].to_vec(), 1.1).is_err());
    let mut unsorted = rows();
    unsorted.swap(0, 2);
    assert!(DescentSchedule::new(unsorted, 1.1).is_err());
    assert!(DescentSchedule::new(rows(), 1.1).is_ok());
}

#[test]
fn schedule_rejects_a_non_physical_gamma() {
    // The shock jump divides by gamma_eff - 1, so gamma at or below 1 must fail at build
    // time, not mid-run when the shock model is first evaluated.
    assert!(DescentSchedule::new(rows(), 1.0).is_err());
    assert!(DescentSchedule::new(rows(), 0.9).is_err());
    assert!(DescentSchedule::new(rows(), f64::NAN).is_err());
    assert!(DescentSchedule::new(rows(), f64::INFINITY).is_err());
    assert!(DescentSchedule::new(rows(), 1.1).is_ok(), "a valid gamma");
}

#[test]
fn schedule_rejects_non_positive_or_non_finite_rows() {
    // A zero temperature row later divides to an invalid Mach input.
    let mut zero_temperature = rows();
    zero_temperature[1].temperature = 0.0;
    assert!(DescentSchedule::new(zero_temperature, 1.1).is_err());

    let mut negative_density = rows();
    negative_density[0].n_tot = -1.0e20;
    assert!(DescentSchedule::new(negative_density, 1.1).is_err());

    let mut zero_sound_speed = rows();
    zero_sound_speed[2].sound_speed = 0.0;
    assert!(DescentSchedule::new(zero_sound_speed, 1.1).is_err());

    let mut nan_altitude = rows();
    nan_altitude[2].altitude_m = f64::NAN;
    assert!(DescentSchedule::new(nan_altitude, 1.1).is_err());

    assert!(DescentSchedule::new(rows(), 1.1).is_ok(), "a valid table");
}

#[test]
fn schedule_interpolates_and_clamps() {
    let s = DescentSchedule::new(rows(), 1.1).unwrap();

    // Clamped at both ends.
    assert_eq!(s.sample(10_000.0).n_tot, 8.0e23);
    assert_eq!(s.sample(200_000.0).n_tot, 7.0e19);

    // Midpoint of the 30-61 km segment interpolates linearly.
    let mid = s.sample(45_500.0);
    let expected = 0.5 * (8.0e23 + 1.3e21);
    assert!((mid.n_tot - expected).abs() / expected < 1e-12);
    assert!((mid.temperature - 238.0).abs() < 1e-9);
}

#[test]
fn builder_requires_every_section_and_converts_the_seed() {
    let trunc = Truncation::<f64>::by_bond(16).unwrap();

    // Missing flight_dt fails.
    let missing = CfdConfigBuilder::compressible_march::<f64>("x")
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 2.0, 1.1, trunc)
        .seed_fn(|_, _| (1.0, 0.1, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(2))
        .reference(REFERENCE.0, REFERENCE.1, REFERENCE.2)
        .build();
    assert!(missing.is_err());

    // The full build succeeds and carries the sections.
    let cfg = CfdConfigBuilder::compressible_march::<f64>("descent")
        .grid(3, 3, 0.125, 0.125)
        .solver(0.002, 2.0, 1.1, trunc)
        .flight_dt(0.05)
        .seed_fn(|_, _| (1.0, 0.1, 0.0, 1.0))
        .unwrap()
        .stop(MarchStop::Fixed(2))
        .observe(QttObserve::default().electron_density())
        .schedule(DescentSchedule::new(rows(), 1.1).unwrap())
        .reference(REFERENCE.0, REFERENCE.1, REFERENCE.2)
        .build()
        .unwrap();
    assert_eq!(cfg.name(), "descent");
    assert_eq!(cfg.modes(), (3, 3));
    assert_eq!(cfg.dt_flight(), 0.05);
    assert!(cfg.schedule().is_some());
    assert_eq!(cfg.reference().t_ref(), 8_044.0);
}

#[test]
fn seed_fn_requires_grid_and_solver_first() {
    let err = CfdConfigBuilder::compressible_march::<f64>("seed_fn_ordering")
        .seed_fn(|_, _| (1.0, 0.0, 0.0, 1.0));
    assert!(err.is_err());
}

// ── PlumeImprint construction: the imprint spec validates its own geometry ──

/// The valid imprint's arguments, so each rejection test varies exactly one of them.
fn imprint_args() -> (f64, usize, f64, f64, f64, f64, [f64; 4], f64) {
    (0.05, 4, 0.72, 0.5, 1.0, 4.0, [1.0, -0.5, 0.0, 2.0], 0.002)
}

#[test]
fn imprint_construction_accepts_a_valid_spec() {
    let a = imprint_args();
    let spec = PlumeImprint::new(a.0, a.1, a.2, a.3, a.4, a.5, a.6, a.7).expect("valid spec");
    assert_eq!(spec.throttle_tolerance(), a.0);
    assert_eq!(spec.max_refreshes(), a.1);
    assert_eq!(spec.face_x(), a.2);
    assert_eq!(spec.axis_y(), a.3);
    assert_eq!(spec.smoothing_cells(), a.4);
    assert_eq!(spec.domain_m(), a.5);
    assert_eq!(spec.target(), a.6);
    assert_eq!(spec.eta(), a.7);
}

#[test]
fn imprint_construction_refuses_a_nonpositive_scale() {
    let a = imprint_args();
    // A zero tolerance would refresh the mask on every step.
    assert!(PlumeImprint::new(0.0, a.1, a.2, a.3, a.4, a.5, a.6, a.7).is_err());
    assert!(PlumeImprint::new(f64::NAN, a.1, a.2, a.3, a.4, a.5, a.6, a.7).is_err());
    // Smoothing skirt, domain width, and penalization strength are all positive scales.
    assert!(PlumeImprint::new(a.0, a.1, a.2, a.3, 0.0, a.5, a.6, a.7).is_err());
    assert!(PlumeImprint::new(a.0, a.1, a.2, a.3, a.4, -1.0, a.6, a.7).is_err());
    assert!(PlumeImprint::new(a.0, a.1, a.2, a.3, a.4, a.5, a.6, 0.0).is_err());
}

#[test]
fn imprint_construction_refuses_a_placement_off_the_unit_square() {
    let a = imprint_args();
    // The mask is built on the unit square, so a face or axis outside it places nothing.
    assert!(PlumeImprint::new(a.0, a.1, 1.5, a.3, a.4, a.5, a.6, a.7).is_err());
    assert!(PlumeImprint::new(a.0, a.1, -0.1, a.3, a.4, a.5, a.6, a.7).is_err());
    assert!(PlumeImprint::new(a.0, a.1, a.2, 1.5, a.4, a.5, a.6, a.7).is_err());
    assert!(PlumeImprint::new(a.0, a.1, a.2, f64::NAN, a.4, a.5, a.6, a.7).is_err());
}

#[test]
fn imprint_construction_refuses_a_nonphysical_target_state() {
    let a = imprint_args();
    // The marcher refuses a non-positive density the moment it sees one.
    assert!(PlumeImprint::new(a.0, a.1, a.2, a.3, a.4, a.5, [0.0, -0.5, 0.0, 2.0], a.7).is_err());
    assert!(PlumeImprint::new(a.0, a.1, a.2, a.3, a.4, a.5, [-1.0, -0.5, 0.0, 2.0], a.7).is_err());
    // Every component must be finite.
    assert!(
        PlumeImprint::new(a.0, a.1, a.2, a.3, a.4, a.5, [1.0, f64::NAN, 0.0, 2.0], a.7).is_err()
    );
}
