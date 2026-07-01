/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use chrono::NaiveDate;
use deep_causality_file::{ClockData, SatId};

fn sample() -> ClockData<f64> {
    let ts = NaiveDate::from_ymd_opt(2016, 7, 1)
        .unwrap()
        .and_hms_opt(12, 30, 15)
        .unwrap();
    ClockData::new(ts, SatId::E14, 0.000_123_456)
}

#[test]
fn test_getters() {
    let c = sample();
    assert_eq!(
        c.timestamp(),
        NaiveDate::from_ymd_opt(2016, 7, 1)
            .unwrap()
            .and_hms_opt(12, 30, 15)
            .unwrap()
    );
    assert_eq!(c.sat_id(), SatId::E14);
    assert_eq!(c.bias_s(), 0.000_123_456);
}

#[test]
fn test_display() {
    let c = sample();
    let s = format!("{c}");
    assert!(s.starts_with("ClockData: timestamp: 2016-07-01 12:30:15"));
    assert!(s.contains("Satellite ID: Galileo/E14"));
    assert!(s.contains("Bias (Sec.): 0.000123456"));
}

#[test]
fn test_clone_and_debug() {
    let c = sample();
    let cloned = c.clone();
    assert_eq!(cloned.bias_s(), c.bias_s());
    assert_eq!(cloned.sat_id(), c.sat_id());
    assert!(format!("{c:?}").contains("ClockData"));
}
