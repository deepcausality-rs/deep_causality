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

fn one_of_each() -> [SpaceTimeRecord; 4] {
    [
        SpaceTimeRecord::Galilean {
            t: 20.0,
            x: 17.0,
            y: 18.0,
            z: 19.0,
            scale: TimeScale::Minute,
        },
        SpaceTimeRecord::Newtonian {
            t: 4.0,
            x: 1.0,
            y: 2.0,
            z: 3.0,
            scale: TimeScale::Second,
        },
        SpaceTimeRecord::Minkowski {
            t: 8.0,
            x: 5.0,
            y: 6.0,
            z: 7.0,
            scale: TimeScale::Nanoseconds,
        },
        SpaceTimeRecord::Tangent {
            t: 12.0,
            x: 9.0,
            y: 10.0,
            z: 11.0,
            dt: 13.0,
            dx: 14.0,
            dy: 15.0,
            dz: 16.0,
            metric: metric(),
        },
    ]
}

#[test]
fn test_four_variants_with_distinct_names() {
    let names: Vec<&str> = one_of_each()
        .iter()
        .map(SpaceTimeRecord::kind_name)
        .collect();
    assert_eq!(names, ["Galilean", "Newtonian", "Minkowski", "Tangent"]);
}

#[test]
fn test_tangent_keeps_every_field_and_is_copy() {
    let [.., tangent] = one_of_each();
    let copy = tangent;
    assert_eq!(tangent, copy);
    let SpaceTimeRecord::Tangent {
        t,
        x,
        y,
        z,
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
        [t, x, y, z, dt, dx, dy, dz],
        [12.0, 9.0, 10.0, 11.0, 13.0, 14.0, 15.0, 16.0]
    );
    // 4 * 3 + 2 and 4 * 2 + 3: the two positions differ, so a transposed read is caught.
    assert_eq!(metric[3][2], 14.0);
    assert_eq!(metric[2][3], 11.0);
    assert_eq!(metric[0][0], 0.0);
}

#[test]
fn test_every_position_and_scale_is_kept() {
    let [galilean, newtonian, minkowski, _] = one_of_each();
    let SpaceTimeRecord::Galilean { t, x, y, z, scale } = galilean else {
        panic!("a Galilean record");
    };
    assert_eq!([t, x, y, z], [20.0, 17.0, 18.0, 19.0]);
    assert_eq!(scale, TimeScale::Minute);
    let SpaceTimeRecord::Newtonian { t, x, y, z, scale } = newtonian else {
        panic!("a Newtonian record");
    };
    assert_eq!([t, x, y, z], [4.0, 1.0, 2.0, 3.0]);
    assert_eq!(scale, TimeScale::Second);
    let SpaceTimeRecord::Minkowski { t, x, y, z, scale } = minkowski else {
        panic!("a Minkowski record");
    };
    assert_eq!([t, x, y, z], [8.0, 5.0, 6.0, 7.0]);
    assert_eq!(scale, TimeScale::Nanoseconds);
    assert!(format!("{minkowski:?}").contains("Nanoseconds"));
}

#[test]
fn test_equal_coordinates_under_different_variants() {
    let at = |make: fn(f64) -> SpaceTimeRecord| make(1.0);
    let galilean = at(|v| SpaceTimeRecord::Galilean {
        t: v,
        x: v,
        y: v,
        z: v,
        scale: TimeScale::Second,
    });
    let newtonian = at(|v| SpaceTimeRecord::Newtonian {
        t: v,
        x: v,
        y: v,
        z: v,
        scale: TimeScale::Second,
    });
    let minkowski = at(|v| SpaceTimeRecord::Minkowski {
        t: v,
        x: v,
        y: v,
        z: v,
        scale: TimeScale::Second,
    });
    // The same numbers name events of different geometries, so the records differ.
    assert_ne!(galilean, newtonian);
    assert_ne!(newtonian, minkowski);
    assert_ne!(galilean, minkowski);
}

#[test]
fn test_zero_time_and_negative_velocity() {
    let record = SpaceTimeRecord::Tangent {
        t: 0.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
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
    let nan = SpaceTimeRecord::Newtonian {
        t: f64::NAN,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        scale: TimeScale::Second,
    };
    assert_ne!(nan, nan);
}
