/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality::*;

fn minkowski_with_scale(scale: TimeScale) -> MinkowskiSpacetime {
    MinkowskiSpacetime::new(1, 1.0, 2.0, 3.0, 1.0, scale)
}

#[test]
fn test_space_temporal_interval_position() {
    let s = MinkowskiSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    assert_eq!(SpaceTemporalInterval::position(&s), [1.0, 2.0, 3.0]);
}

#[test]
fn test_space_temporal_interval_time_all_scales() {
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Nanoseconds)),
        1.0 / 1_000_000_000.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Microseconds)),
        1.0 / 1_000_000.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Millisecond)),
        1.0 / 1_000.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Second)),
        1.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Minute)),
        60.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Hour)),
        3_600.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Day)),
        86_400.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Week)),
        604_800.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Month)),
        2_629_746.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Quarter)),
        7_889_238.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Year)),
        31_556_952.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::NoScale)),
        1.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Steps)),
        1.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&minkowski_with_scale(TimeScale::Symbolic)),
        1.0
    );
}
