// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality::types::context_types::time_scale::TimeScale;

#[test]
fn test_time_scale() {
    let ts = TimeScale::NoScale;
    assert_eq!(ts, TimeScale::NoScale);
    assert_eq!(ts.to_string(), "NoScale");

    let ts = TimeScale::Second;
    assert_eq!(ts, TimeScale::Second);
    assert_eq!(ts.to_string(), "Second");

    let ts = TimeScale::Minute;
    assert_eq!(ts, TimeScale::Minute);
    assert_eq!(ts.to_string(), "Minute");

    let ts = TimeScale::Hour;
    assert_eq!(ts, TimeScale::Hour);
    assert_eq!(ts.to_string(), "Hour");

    let ts = TimeScale::Day;
    assert_eq!(ts, TimeScale::Day);
    assert_eq!(ts.to_string(), "Day");

    let ts = TimeScale::Week;
    assert_eq!(ts, TimeScale::Week);
    assert_eq!(ts.to_string(), "Week");

    let ts = TimeScale::Month;
    assert_eq!(ts, TimeScale::Month);
    assert_eq!(ts.to_string(), "Month");

    let ts = TimeScale::Quarter;
    assert_eq!(ts, TimeScale::Quarter);
    assert_eq!(ts.to_string(), "Quarter");

    let ts = TimeScale::Year;
    assert_eq!(ts, TimeScale::Year);
    assert_eq!(ts.to_string(), "Year");
}
