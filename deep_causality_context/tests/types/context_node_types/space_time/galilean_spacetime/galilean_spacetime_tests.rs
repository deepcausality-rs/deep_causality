/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_context::*;

#[test]
fn test_identifiable_trait() {
    let s = GalileanSpacetime::new(42, 1.0, 2.0, 3.0, 1_000_000.0, TimeScale::Second);
    assert_eq!(s.id(), 42);
}

#[test]
fn test_coordinate_trait() {
    let s = GalileanSpacetime::new(1, 1.0, 2.0, 3.0, 0.0, TimeScale::Second);

    // Time first: 0 => t, 1 => x, 2 => y, 3 => z.
    assert_eq!(s.dimension(), 4);
    assert_eq!(*s.coordinate(0).unwrap(), 0.0);
    assert_eq!(*s.coordinate(1).unwrap(), 1.0);
    assert_eq!(*s.coordinate(2).unwrap(), 2.0);
    assert_eq!(*s.coordinate(3).unwrap(), 3.0);
}

#[test]
fn test_coordinate_trait_out_of_bounds() {
    let s = GalileanSpacetime::new(1, 1.0, 2.0, 3.0, 0.0, TimeScale::Second);
    let res = s.coordinate(4);
    assert!(res.is_err());
}

#[test]
fn test_display_trait() {
    let s = GalileanSpacetime::new(1, 1.0, 2.0, 3.0, 1_000_000.0, TimeScale::Second);
    let formatted = format!("{s}");
    dbg!(&formatted);
    assert!(formatted.contains("GalileanSpacetime"));
    assert!(formatted.contains("x=1.0"));
    assert!(formatted.contains("t=1000000"));
}

#[test]
fn test_metric_trait() {
    let a: GalileanSpacetime<f64> =
        GalileanSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let b = GalileanSpacetime::new(2, 3.0, 4.0, 0.0, 0.0, TimeScale::Second);

    let dist = a.distance(&b);
    assert_eq!(dist, 5.0);
}

#[test]
fn test_events_at_different_times_have_no_distance() {
    // Galilean spacetime has no spatial distance between non-simultaneous events: the same
    // coordinates a minute later are not "the same place".
    let a: GalileanSpacetime<f64> =
        GalileanSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    let later = GalileanSpacetime::new(2, 0.0, 0.0, 0.0, 60.0, TimeScale::Second);
    assert!(a.distance(&later).is_nan());
    assert!(later.distance(&a).is_nan());
}

#[test]
fn test_simultaneity_compares_instants_across_time_scales() {
    let a: GalileanSpacetime<f64> =
        GalileanSpacetime::new(1, 0.0, 0.0, 0.0, 60.0, TimeScale::Second);
    let b = GalileanSpacetime::new(2, 3.0, 4.0, 0.0, 1.0, TimeScale::Minute);
    let c = GalileanSpacetime::new(3, 3.0, 4.0, 0.0, 60_000.0, TimeScale::Millisecond);
    assert_eq!(a.is_simultaneous_with(&b), Some(true));
    assert_eq!(b.is_simultaneous_with(&c), Some(true));
    assert_eq!(a.distance(&b), 5.0);
    assert_eq!(a.distance(&c), 5.0);
}

#[test]
fn test_simultaneity_of_scales_without_a_duration() {
    // Steps compare with steps directly; a step and a second cannot be compared.
    let a: GalileanSpacetime<f64> = GalileanSpacetime::new(1, 0.0, 0.0, 0.0, 3.0, TimeScale::Steps);
    let b = GalileanSpacetime::new(2, 1.0, 0.0, 0.0, 3.0, TimeScale::Steps);
    let c = GalileanSpacetime::new(3, 1.0, 0.0, 0.0, 4.0, TimeScale::Steps);
    let d = GalileanSpacetime::new(4, 1.0, 0.0, 0.0, 3.0, TimeScale::Second);
    let e = GalileanSpacetime::new(5, 1.0, 0.0, 0.0, 3.0, TimeScale::Symbolic);
    assert_eq!(a.is_simultaneous_with(&b), Some(true));
    assert_eq!(a.is_simultaneous_with(&c), Some(false));
    assert_eq!(a.is_simultaneous_with(&d), None);
    assert_eq!(a.is_simultaneous_with(&e), None);
    assert_eq!(a.distance(&b), 1.0);
    assert!(a.distance(&c).is_nan());
    assert!(a.distance(&d).is_nan());
}

#[test]
fn test_every_physical_time_scale_converts_to_seconds() {
    // Two of each unit against the same instant in seconds; every product and quotient here is
    // exact in f64, so a wrong factor fails the comparison. Month, Quarter and Year are Gregorian
    // means: a year is 365.2425 days, 31 556 952 s.
    let cases = [
        (TimeScale::Nanoseconds, 2_000_000_000.0, 2.0),
        (TimeScale::Microseconds, 2_000_000.0, 2.0),
        (TimeScale::Millisecond, 2_000.0, 2.0),
        (TimeScale::Second, 2.0, 2.0),
        (TimeScale::Minute, 2.0, 120.0),
        (TimeScale::Hour, 2.0, 7_200.0),
        (TimeScale::Day, 2.0, 172_800.0),
        (TimeScale::Week, 2.0, 1_209_600.0),
        (TimeScale::Month, 2.0, 5_259_492.0),
        (TimeScale::Quarter, 2.0, 15_778_476.0),
        (TimeScale::Year, 2.0, 63_113_904.0),
    ];
    for (scale, t, seconds) in cases {
        let e = GalileanSpacetime::new(1, 0.0, 0.0, 0.0, t, scale);
        let same = GalileanSpacetime::new(2, 0.0, 0.0, 0.0, seconds, TimeScale::Second);
        let off = GalileanSpacetime::new(3, 0.0, 0.0, 0.0, seconds + 1.0, TimeScale::Second);
        assert_eq!(e.is_simultaneous_with(&same), Some(true), "{scale:?}");
        assert_eq!(e.is_simultaneous_with(&off), Some(false), "{scale:?}");
    }
}

#[test]
fn test_temporal_trait() {
    let s = GalileanSpacetime::new(1, 0.0, 0.0, 0.0, 123456.0, TimeScale::Second);

    assert_eq!(s.time_scale(), TimeScale::Second);
    assert_eq!(s.time_unit(), 123456.0);
}

#[test]
fn test_space_temporal_trait() {
    let s = GalileanSpacetime::new(1, 9.0, 8.0, 7.0, 654321.0, TimeScale::Second);

    assert_eq!(s.t(), &654321.0);
    assert_eq!(*s.coordinate(0).unwrap(), 654321.0);
    assert_eq!(*s.coordinate(1).unwrap(), 9.0);
}

#[test]
fn test_simultaneity_and_distance_at_every_precision() {
    // Row K. Two minutes and 120 s are the same instant, exactly, at f32 and Float106 too.
    let a = GalileanSpacetime::new(1, 0.0_f32, 0.0, 0.0, 2.0, TimeScale::Minute);
    let b = GalileanSpacetime::new(2, 3.0_f32, 4.0, 0.0, 120.0, TimeScale::Second);
    assert_eq!(a.is_simultaneous_with(&b), Some(true));
    assert_eq!(a.distance(&b), 5.0);

    let w = deep_causality_num::Float106::from;
    let a = GalileanSpacetime::new(1, w(0.0), w(0.0), w(0.0), w(2.0), TimeScale::Minute);
    let b = GalileanSpacetime::new(2, w(3.0), w(4.0), w(0.0), w(120.0), TimeScale::Second);
    let later = GalileanSpacetime::new(3, w(3.0), w(4.0), w(0.0), w(121.0), TimeScale::Second);
    assert_eq!(a.is_simultaneous_with(&b), Some(true));
    assert_eq!(f64::from(a.distance(&b)), 5.0);
    assert!(f64::from(a.distance(&later)).is_nan());
}

#[test]
fn test_distance_between_two_events_away_from_the_origin() {
    // Row C: neither event at the origin, so a sum of coordinates cannot pass for a difference.
    // (1, 2, 3) to (4, 6, 15) is the 3-4-12 triangle, 13 m.
    let a: GalileanSpacetime<f64> =
        GalileanSpacetime::new(1, 1.0, 2.0, 3.0, 7.0, TimeScale::Second);
    let b = GalileanSpacetime::new(2, 4.0, 6.0, 15.0, 7.0, TimeScale::Second);
    assert_eq!(a.distance(&b), 13.0);
    assert_eq!(b.distance(&a), 13.0);
}
