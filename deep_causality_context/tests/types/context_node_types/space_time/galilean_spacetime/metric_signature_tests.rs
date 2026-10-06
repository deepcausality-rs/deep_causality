/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::{Coordinate, Distance, GalileanSpacetime, MetricSignature, TimeScale};
use deep_causality_metric::{Metric, detect_convention, is_lorentzian};

/// An event whose coordinates, in the type's order `t, x, y, z`, are `c`.
fn event(c: [f64; 4]) -> GalileanSpacetime<f64> {
    // The constructor takes `id, x, y, z, t, scale`.
    GalileanSpacetime::new(1, c[1], c[2], c[3], c[0], TimeScale::Second)
}

#[test]
fn test_reports_the_spatial_metric_signature_time_first() {
    // h^ab of a classical spacetime has signature (0, 1, 1, 1).
    let e = event([4.0, 1.0, 2.0, 3.0]);
    assert_eq!(e.metric(), Metric::PGA(4));
    let signs: Vec<i32> = (0..4).map(|i| e.metric().sign_of_sq(i)).collect();
    assert_eq!(signs, [0, 1, 1, 1]);
    assert_eq!(e.metric().dimension(), e.dimension());
}

#[test]
fn test_spatial_axes_carry_distance_and_the_time_axis_leaves_it_undefined() {
    // At one instant a displacement along a spatial axis is a distance; a displacement along the
    // time axis makes the events non-simultaneous, and Galilean spacetime gives them none.
    let origin = event([0.0; 4]);
    for axis in 1..4 {
        let mut c = [0.0; 4];
        c[axis] = 2.0;
        assert_eq!(origin.distance(&event(c)), 2.0, "axis {axis}");
    }
    assert!(origin.distance(&event([2.0, 0.0, 0.0, 0.0])).is_nan());
}

#[test]
fn test_the_distance_squared_is_the_spatial_quadratic_form_at_one_instant() {
    let a = event([7.0, 0.0, 0.0, 0.0]);
    let b = event([7.0, 3.0, 4.0, 12.0]);
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
fn test_it_is_not_lorentzian_and_has_no_sign_convention() {
    let e = event([0.0; 4]);
    assert!(!is_lorentzian(&e.metric()));
    assert_eq!(detect_convention(&e.metric()), None);
}
