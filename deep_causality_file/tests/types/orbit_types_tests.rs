/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use chrono::NaiveDate;
use deep_causality_file::{OrbitData, SatId};

fn sample() -> OrbitData<f64> {
    let ts = NaiveDate::from_ymd_opt(2016, 7, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    // A 3-4-12 triple scaled by 1000 → radius 13000.
    OrbitData::new(ts, SatId::E14, 3000.0, 4000.0, 12000.0)
}

#[test]
fn test_getters() {
    let o = sample();
    assert_eq!(
        o.timestamp(),
        NaiveDate::from_ymd_opt(2016, 7, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
    );
    assert_eq!(o.sat_id(), &SatId::E14);
    assert_eq!(o.x_m(), 3000.0);
    assert_eq!(o.y_m(), 4000.0);
    assert_eq!(o.z_m(), 12000.0);
}

#[test]
fn test_radius_m() {
    let o = sample();
    // sqrt(3000^2 + 4000^2 + 12000^2) = sqrt(169_000_000) = 13000.
    assert_eq!(o.radius_m(), 13000.0);
}

#[test]
fn test_display() {
    let o = sample();
    let s = format!("{o}");
    assert!(s.starts_with("OrbitData: timestamp: 2016-07-01 00:00:00"));
    assert!(s.contains("Satellite ID: Galileo/E14"));
    assert!(s.contains("X: 3000"));
    assert!(s.contains("Y: 4000"));
    assert!(s.contains("Z: 12000"));
}

#[test]
fn test_clone_and_debug() {
    let o = sample();
    let cloned = o.clone();
    assert_eq!(cloned.radius_m(), o.radius_m());
    assert!(format!("{o:?}").contains("OrbitData"));
}
