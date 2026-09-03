/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `GnssDataResult<R>` alias carries no behaviour of its own; this test simply pins that the
//! public alias names the expected `Result<(Vec<ClockData>, Vec<OrbitData>), DataLoadingError>`.

use chrono::NaiveDate;
use deep_causality_file::{ClockData, GnssDataResult, OrbitData, SatId};

/// Build the pair behind the public alias. Returning `GnssDataResult` (rather than an inline `Ok`)
/// both pins the alias shape and keeps the value out of clippy's literal-unwrap sight.
fn build() -> GnssDataResult<f64> {
    let ts = NaiveDate::from_ymd_opt(2016, 7, 1)
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let clocks = vec![ClockData::new(ts, SatId::E14, 0.1)];
    let orbits = vec![OrbitData::new(ts, SatId::E14, 1.0, 2.0, 3.0)];
    Ok((clocks, orbits))
}

#[test]
fn test_alias_holds_clock_and_orbit_pair() {
    let (c, o) = build().unwrap();
    assert_eq!(c.len(), 1);
    assert_eq!(o.len(), 1);
}
