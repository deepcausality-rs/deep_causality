/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{
    Coordinate, Distance, MetricSignature, NewtonianSpacetime, TimeScale,
};
use deep_causality_metric::{Metric, detect_convention, is_lorentzian};

/// An event whose coordinates, in the type's order `t, x, y, z`, are `c`.
fn event(c: [f64; 4]) -> NewtonianSpacetime<f64> {
    // The constructor takes `id, x, y, z, t, scale`.
    NewtonianSpacetime::new(1, c[1], c[2], c[3], c[0], TimeScale::Second)
}

#[test]
fn test_reports_a_null_time_axis_and_flat_space() {
    // The spatial metric relative to absolute space, (0, 1, 1, 1), time first.
    let e = event([4.0, 1.0, 2.0, 3.0]);
    assert_eq!(e.metric(), Metric::PGA(4));
    assert_eq!(e.metric().signature(), (3, 0, 1));
    let signs: Vec<i32> = (0..4).map(|i| e.metric().sign_of_sq(i)).collect();
    assert_eq!(signs, [0, 1, 1, 1]);
    assert_eq!(e.metric().dimension(), e.dimension());
}

#[test]
fn test_the_signature_agrees_with_the_distance_on_every_axis() {
    // A displacement along one axis changes the distance exactly when the signature gives that
    // axis a non-zero square.
    let origin = event([0.0; 4]);
    let metric = origin.metric();
    for axis in 0..origin.dimension() {
        let mut c = [0.0; 4];
        c[axis] = 2.0;
        assert_eq!(
            origin.distance(&event(c)) != 0.0,
            metric.sign_of_sq(axis) != 0,
            "axis {axis}"
        );
    }
}

#[test]
fn test_the_distance_squared_is_the_signature_quadratic_form() {
    // Σᵢ sign(eᵢ²)·Δᵢ² over a displacement on all four axes. The time term drops and the spatial
    // part is a 3-4-12 triangle, so the distance is exactly 13; a (+,+,+,+) form would add
    // Δt² = 25 and give √194.
    let a = event([0.0; 4]);
    let b = event([5.0, 3.0, 4.0, 12.0]);
    let metric = a.metric();
    let form: f64 = (0..a.dimension())
        .map(|i| {
            let d = b.coordinate(i).unwrap() - a.coordinate(i).unwrap();
            f64::from(metric.sign_of_sq(i)) * d * d
        })
        .sum();
    assert_eq!(a.distance(&b), 13.0);
    assert_eq!(a.distance(&b) * a.distance(&b), form);
}

#[test]
fn test_the_signature_does_not_depend_on_the_coordinates() {
    let here = NewtonianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let there = NewtonianSpacetime::new(2, 9.0, -3.0, 7.5, 42.0, TimeScale::Millisecond);
    assert_eq!(here.metric(), there.metric());
}

#[test]
fn test_it_is_not_lorentzian_and_has_no_sign_convention() {
    // The time axis is null rather than of opposite sign to space.
    let e = event([0.0; 4]);
    assert!(!is_lorentzian(&e.metric()));
    assert_eq!(detect_convention(&e.metric()), None);
}
