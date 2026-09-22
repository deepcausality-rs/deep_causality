/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructors, read back by pattern. The metric
//! tensor's entries are `4 * row + column`, so every entry is distinct and a transposed read would
//! be caught (`metric[3][2]` is 14, `metric[2][3]` is 11).
//!
//! Corner cases (rows A to K): A/B n/a; C coinciding coordinates across variants in
//! `test_equal_coordinates_under_different_variants`; D index degeneracy: the metric is read at an
//! off-diagonal position where row and column differ, `test_tangent_keeps_every_field_and_is_copy`;
//! E n/a; F zero time in `test_zero_time_and_negative_velocity`; G negative velocity in the same
//! test; H n/a; I non-finite in `test_non_finite_time`; J/K n/a.

use deep_causality_context_store::{SpaceTimeRecord, TimeScale};

fn metric() -> [[f64; 4]; 4] {
    let mut m = [[0.0; 4]; 4];
    for (i, row) in m.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = (i * 4 + j) as f64;
        }
    }
    m
}

fn one_of_each() -> [SpaceTimeRecord; 3] {
    [
        SpaceTimeRecord::Euclidean {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            t: 4.0,
            scale: TimeScale::Second,
        },
        SpaceTimeRecord::Lorentzian {
            x: 5.0,
            y: 6.0,
            z: 7.0,
            t: 8.0,
            scale: TimeScale::Nanoseconds,
        },
        SpaceTimeRecord::Tangent {
            x: 9.0,
            y: 10.0,
            z: 11.0,
            t: 12.0,
            dt: 13.0,
            dx: 14.0,
            dy: 15.0,
            dz: 16.0,
            metric: metric(),
        },
    ]
}

#[test]
fn test_three_variants_with_distinct_names() {
    let names: Vec<&str> = one_of_each()
        .iter()
        .map(SpaceTimeRecord::kind_name)
        .collect();
    assert_eq!(names, ["Euclidean", "Lorentzian", "Tangent"]);
}

#[test]
fn test_tangent_keeps_every_field_and_is_copy() {
    let [.., tangent] = one_of_each();
    let copy = tangent;
    assert_eq!(tangent, copy);
    let SpaceTimeRecord::Tangent {
        x,
        y,
        z,
        t,
        dt,
        dx,
        dy,
        dz,
        metric,
    } = copy
    else {
        panic!("a Tangent record");
    };
    assert_eq!(
        [x, y, z, t, dt, dx, dy, dz],
        [9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0]
    );
    // 4 * 3 + 2 and 4 * 2 + 3: the two positions differ, so a transposed read is caught.
    assert_eq!(metric[3][2], 14.0);
    assert_eq!(metric[2][3], 11.0);
    assert_eq!(metric[0][0], 0.0);
}

#[test]
fn test_scale_is_kept() {
    let [euclidean, lorentzian, _] = one_of_each();
    let SpaceTimeRecord::Euclidean { scale, .. } = euclidean else {
        panic!("a Euclidean record");
    };
    assert_eq!(scale, TimeScale::Second);
    let SpaceTimeRecord::Lorentzian { scale, .. } = lorentzian else {
        panic!("a Lorentzian record");
    };
    assert_eq!(scale, TimeScale::Nanoseconds);
    assert!(format!("{lorentzian:?}").contains("Nanoseconds"));
}

#[test]
fn test_equal_coordinates_under_different_variants() {
    let euclidean = SpaceTimeRecord::Euclidean {
        x: 1.0,
        y: 1.0,
        z: 1.0,
        t: 1.0,
        scale: TimeScale::Second,
    };
    let lorentzian = SpaceTimeRecord::Lorentzian {
        x: 1.0,
        y: 1.0,
        z: 1.0,
        t: 1.0,
        scale: TimeScale::Second,
    };
    assert_ne!(euclidean, lorentzian);
}

#[test]
fn test_zero_time_and_negative_velocity() {
    let record = SpaceTimeRecord::Tangent {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        t: 0.0,
        dt: 1.0,
        dx: -3.0,
        dy: 0.0,
        dz: 0.0,
        metric: [[0.0; 4]; 4],
    };
    let SpaceTimeRecord::Tangent { t, dx, .. } = record else {
        panic!("a Tangent record");
    };
    assert_eq!(t, 0.0);
    assert_eq!(dx, -3.0);
}

#[test]
fn test_non_finite_time() {
    let nan = SpaceTimeRecord::Euclidean {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        t: f64::NAN,
        scale: TimeScale::Second,
    };
    assert_ne!(nan, nan);
}
