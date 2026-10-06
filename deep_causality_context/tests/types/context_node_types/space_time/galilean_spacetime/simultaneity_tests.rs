/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt::Debug;
use deep_causality_algebra::RealField;
use deep_causality_context::{Distance, GalileanSpacetime, TimeScale};
use deep_causality_num::{Float106, FromPrimitive, lift_count};

/// One instant in several scales: in each `(scale, factor, denominator)`, the time is
/// `factor * n / denominator` for the same `n`.
type Row = &'static [(TimeScale, u64, u64)];

/// `n / 100` h.
const HOURS: Row = &[
    (TimeScale::Hour, 1, 100),
    (TimeScale::Minute, 3, 5),
    (TimeScale::Second, 36, 1),
];

/// `n / 10` d.
const DAYS: Row = &[
    (TimeScale::Day, 1, 10),
    (TimeScale::Hour, 12, 5),
    (TimeScale::Minute, 144, 1),
    (TimeScale::Second, 8_640, 1),
];

/// `n / 100` ms.
const MILLISECONDS: Row = &[
    (TimeScale::Millisecond, 1, 100),
    (TimeScale::Microseconds, 10, 1),
    (TimeScale::Nanoseconds, 10_000, 1),
    (TimeScale::Second, 1, 100_000),
];

/// `n / 10` weeks.
const WEEKS: Row = &[
    (TimeScale::Week, 1, 10),
    (TimeScale::Day, 7, 10),
    (TimeScale::Hour, 84, 5),
    (TimeScale::Minute, 1_008, 1),
    (TimeScale::Second, 60_480, 1),
];

/// `n / 10` months; a month is 2 629 746 s.
const MONTHS: Row = &[
    (TimeScale::Month, 1, 10),
    (TimeScale::Week, 6_957, 16_000),
    (TimeScale::Day, 48_699, 16_000),
    (TimeScale::Second, 1_314_873, 5),
];

/// `n / 10` years; a year is 31 556 952 s.
const YEARS: Row = &[
    (TimeScale::Year, 1, 10),
    (TimeScale::Quarter, 2, 5),
    (TimeScale::Month, 6, 5),
    (TimeScale::Day, 146_097, 4_000),
    (TimeScale::Second, 15_778_476, 5),
];

/// `numerator / denominator` in `R`. While both integers are exact in `R`, for `f32` and `f64`
/// this is the float a decimal literal for the quotient gives.
fn decimal<R: RealField + FromPrimitive>(numerator: u64, denominator: u64) -> R {
    lift_count::<R>(numerator) / lift_count::<R>(denominator)
}

/// An event at time `t` in `scale`, at `(x, y, 0)`.
fn event<R: RealField + FromPrimitive>(t: R, scale: TimeScale, x: R, y: R) -> GalileanSpacetime<R> {
    GalileanSpacetime::new(1, x, y, lift_count(0), t, scale)
}

/// Every pair of times in a row, for `n` in `1..=n_max`, is one instant.
fn assert_rows_are_one_instant<R>(rows: &[Row], n_max: u64)
where
    R: RealField + FromPrimitive + Debug,
{
    let zero: R = lift_count(0);
    for row in rows {
        for n in 1..=n_max {
            let events: Vec<GalileanSpacetime<R>> = row
                .iter()
                .map(|&(scale, factor, denominator)| {
                    event(decimal(factor * n, denominator), scale, zero, zero)
                })
                .collect();
            for a in &events {
                for b in &events {
                    assert_eq!(a.is_simultaneous_with(b), Some(true), "{a:?} {b:?}");
                }
            }
        }
    }
}

#[test]
fn test_a_decimal_time_in_months_is_the_same_instant_in_seconds() {
    // 0.1 months is 262 974.6 s. The product 0.1 * 2 629 746 rounds to the f64 above 262 974.6.
    assert_eq!(0.1_f64 * 2_629_746.0, f64::next_up(262_974.6));
    let month = event(0.1, TimeScale::Month, 0.0, 0.0);
    let second = event(262_974.6, TimeScale::Second, 3.0, 4.0);
    assert_eq!(month.is_simultaneous_with(&second), Some(true));
    assert_eq!(second.is_simultaneous_with(&month), Some(true));
    assert_eq!(month.distance(&second), 5.0);

    // The same instant before the epoch.
    let month = event(-0.1, TimeScale::Month, 0.0, 0.0);
    let second = event(-262_974.6, TimeScale::Second, 3.0, 4.0);
    assert_eq!(month.is_simultaneous_with(&second), Some(true));
    assert_eq!(second.is_simultaneous_with(&month), Some(true));

    // A tenth of a second later is another instant.
    let month = event(0.1_f64, TimeScale::Month, 0.0, 0.0);
    let later = event(262_974.7, TimeScale::Second, 3.0, 4.0);
    assert_eq!(month.is_simultaneous_with(&later), Some(false));
    assert!(month.distance(&later).is_nan());
}

#[test]
fn test_decimal_times_converted_on_both_sides_are_one_instant() {
    // 1.11 h and 66.6 min are both 3 996 s. The two conversions round in opposite directions
    // and land two ulps apart, more than epsilon times either.
    let hours = 1.11_f64 * 3_600.0;
    let minutes = 66.6_f64 * 60.0;
    assert_eq!(minutes.next_up().next_up(), hours);
    assert!(hours - minutes > f64::EPSILON * hours);

    let h = event(1.11, TimeScale::Hour, 0.0, 0.0);
    let min = event(66.6, TimeScale::Minute, 3.0, 4.0);
    assert_eq!(h.is_simultaneous_with(&min), Some(true));
    assert_eq!(min.is_simultaneous_with(&h), Some(true));
    assert_eq!(h.distance(&min), 5.0);
}

#[test]
fn test_every_decimal_time_is_one_instant_in_each_scale() {
    assert_rows_are_one_instant::<f64>(&[HOURS, DAYS, MILLISECONDS, WEEKS, MONTHS, YEARS], 10_000);
}

#[test]
fn test_every_decimal_time_is_one_instant_in_each_scale_in_f32() {
    // Rows whose integers stay below 2^24, exact in f32, for n up to 1 000.
    assert_rows_are_one_instant::<f32>(&[HOURS, DAYS, MILLISECONDS], 1_000);
}

#[test]
fn test_every_decimal_time_is_one_instant_in_each_scale_in_float106() {
    assert_rows_are_one_instant::<Float106>(
        &[HOURS, DAYS, MILLISECONDS, WEEKS, MONTHS, YEARS],
        1_000,
    );
}

#[test]
fn test_cross_scale_times_beyond_the_rounding_are_distinct() {
    // One minute against 60 s plus k ulps of 60, an ulp being 2^-47. The bound
    // epsilon * (60 + 60) is 3.75 ulps, so three ulps are the same instant and four are not.
    let minute = event(1.0, TimeScale::Minute, 0.0, 0.0);
    let mut seconds = 60.0_f64;
    for k in 0..=4 {
        let e = event(seconds, TimeScale::Second, 0.0, 0.0);
        assert_eq!(minute.is_simultaneous_with(&e), Some(k <= 3), "{k} ulps");
        assert_eq!(e.is_simultaneous_with(&minute), Some(k <= 3), "{k} ulps");
        seconds = seconds.next_up();
    }
}

#[test]
fn test_the_instant_zero_is_one_instant_in_two_scales() {
    // The bound is zero here, so only an inclusive comparison holds.
    let minute = event(0.0, TimeScale::Minute, 0.0, 0.0);
    let second = event(0.0, TimeScale::Second, 0.0, 0.0);
    assert_eq!(minute.is_simultaneous_with(&second), Some(true));
}

#[test]
fn test_same_scale_times_one_ulp_apart_are_distinct() {
    // 1.6 months and 1.9 ns each land on one value in seconds with the f64 above them; in their
    // own scale they differ.
    assert_eq!(1.6_f64 * 2_629_746.0, f64::next_up(1.6) * 2_629_746.0);
    assert_eq!(
        1.9_f64 / 1_000_000_000.0,
        f64::next_up(1.9) / 1_000_000_000.0
    );
    for (scale, t) in [(TimeScale::Month, 1.6), (TimeScale::Nanoseconds, 1.9)] {
        let a = event(t, scale, 0.0, 0.0);
        let b = event(f64::next_up(t), scale, 0.0, 0.0);
        assert_eq!(a.is_simultaneous_with(&b), Some(false), "{scale:?}");
        assert!(a.distance(&b).is_nan(), "{scale:?}");
    }
}

#[test]
fn test_simultaneity_across_scales_is_not_transitive() {
    // 262 974.6 s and the next f64 above it are both 0.1 months, and two instants.
    let month = event(0.1, TimeScale::Month, 0.0, 0.0);
    let second = event(262_974.6, TimeScale::Second, 0.0, 0.0);
    let next = event(f64::next_up(262_974.6), TimeScale::Second, 0.0, 0.0);
    assert_eq!(month.is_simultaneous_with(&second), Some(true));
    assert_eq!(month.is_simultaneous_with(&next), Some(true));
    assert_eq!(second.is_simultaneous_with(&next), Some(false));
}

#[test]
fn test_decimal_times_are_one_instant_in_f32() {
    // 0.3 d is 25 920 s and 0.7 months is 1 840 822.2 s; in f32 both conversions round to the
    // float next to the seconds value.
    assert_eq!(0.3_f32 * 86_400.0, f32::next_up(25_920.0));
    assert_eq!(0.7_f32 * 2_629_746.0, f32::next_down(1_840_822.2));
    let cases = [
        (0.3_f32, TimeScale::Day, 25_920.0_f32),
        (0.7, TimeScale::Month, 1_840_822.2),
    ];
    for (t, scale, seconds) in cases {
        let a = event(t, scale, 0.0, 0.0);
        let b = event(seconds, TimeScale::Second, 3.0, 4.0);
        assert_eq!(a.is_simultaneous_with(&b), Some(true), "{scale:?}");
        assert_eq!(b.is_simultaneous_with(&a), Some(true), "{scale:?}");
        assert_eq!(a.distance(&b), 5.0, "{scale:?}");
    }
}

#[test]
fn test_decimal_times_are_one_instant_in_float106() {
    // 0.1 months against 262 974.6 s and 1.11 h against 66.6 min, each decimal a quotient of
    // two integers in Float106.
    let w = Float106::from;
    let cases = [
        (
            decimal(1, 10),
            TimeScale::Month,
            decimal(2_629_746, 10),
            TimeScale::Second,
        ),
        (
            decimal(111, 100),
            TimeScale::Hour,
            decimal(666, 10),
            TimeScale::Minute,
        ),
    ];
    for (t, scale, u, other_scale) in cases {
        let a = event(t, scale, w(0.0), w(0.0));
        let b = event(u, other_scale, w(3.0), w(4.0));
        assert_eq!(a.is_simultaneous_with(&b), Some(true), "{scale:?}");
        assert_eq!(b.is_simultaneous_with(&a), Some(true), "{scale:?}");
        assert_eq!(f64::from(a.distance(&b)), 5.0, "{scale:?}");
    }
}

#[test]
fn test_a_time_that_is_not_finite_names_no_instant() {
    // An infinite time in minutes against 0 s: an inclusive bound would read inf <= inf as
    // simultaneous.
    let zero = event(0.0, TimeScale::Second, 0.0, 0.0);
    for t in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        let e = event(t, TimeScale::Minute, 3.0, 4.0);
        assert_eq!(e.is_simultaneous_with(&zero), None, "{t}");
        assert_eq!(zero.is_simultaneous_with(&e), None, "{t}");
        assert!(e.distance(&zero).is_nan(), "{t}");
        // In one scale as well.
        let same = event(t, TimeScale::Minute, 0.0, 0.0);
        assert_eq!(e.is_simultaneous_with(&same), None, "{t}");
    }
}

#[test]
fn test_a_time_that_overflows_in_seconds_names_no_instant() {
    // 3e38 years is finite in f32; in seconds it is past f32::MAX.
    let years = event(3.0e38_f32, TimeScale::Year, 0.0, 0.0);
    let seconds = event(f32::MAX, TimeScale::Second, 0.0, 0.0);
    assert_eq!(years.is_simultaneous_with(&seconds), None);
    assert!(years.distance(&seconds).is_nan());
    // In its own scale the same time is still an instant.
    assert_eq!(years.is_simultaneous_with(&years), Some(true));
}

#[test]
fn test_a_float106_lifted_from_an_f64_literal_keeps_its_error() {
    // The f64 error of 0.1 is far above the Float106 epsilon, so the bound does not cover it.
    let month = event(
        Float106::from(0.1),
        TimeScale::Month,
        Float106::from(0.0),
        Float106::from(0.0),
    );
    let second = event(
        Float106::from(262_974.6),
        TimeScale::Second,
        Float106::from(0.0),
        Float106::from(0.0),
    );
    assert_eq!(month.is_simultaneous_with(&second), Some(false));
}
