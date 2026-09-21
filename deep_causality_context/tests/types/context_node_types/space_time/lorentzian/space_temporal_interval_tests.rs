/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_context::FloatType;
use deep_causality_context::*;

fn lorentzian_with_scale(scale: TimeScale) -> LorentzianSpacetime<FloatType> {
    LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 1.0, scale)
}

#[test]
fn test_space_temporal_interval_position() {
    let s = LorentzianSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, TimeScale::Second);
    assert_eq!(SpaceTemporalInterval::position(&s), [1.0, 2.0, 3.0]);
}

#[test]
fn test_space_temporal_interval_time_all_scales() {
    // Each arm of the `time()` match converts the raw `t = 1.0` into seconds.
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Nanoseconds)),
        1.0 / 1_000_000_000.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Microseconds)),
        1.0 / 1_000_000.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Millisecond)),
        1.0 / 1_000.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Second)),
        1.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Minute)),
        60.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Hour)),
        3_600.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Day)),
        86_400.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Week)),
        604_800.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Month)),
        2_629_746.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Quarter)),
        7_889_238.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Year)),
        31_556_952.0
    );

    // Non-physical scales return the raw value unchanged.
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::NoScale)),
        1.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Steps)),
        1.0
    );
    assert_eq!(
        SpaceTemporalInterval::time(&lorentzian_with_scale(TimeScale::Symbolic)),
        1.0
    );
}

// =============================================================================
// interval_squared() — both terms live, and the speed of light pinned
// =============================================================================

/// Separations whose Minkowski interval is known in closed form.
///
/// Each row is `(Δt seconds, Δx, Δy, Δz, s²)`. The expectations were evaluated in exact integer
/// arithmetic from `s² = -(c·Δt)² + Δx² + Δy² + Δz²` with `c = 299_792_458 m/s`, not by running
/// the kernel: every term is an integer, so the closed form is decidable by hand.
///
/// `Δy` and `Δz` are distinct and nonzero in every row, and in the first four rows neither the
/// time term nor the space term vanishes, so `c` carries into the answer.
const INTERVAL_TABLE: [(FloatType, FloatType, FloatType, FloatType, FloatType); 5] = [
    // Space-like, space term 5.45x the time term.
    (1.0, 2.0e8, 3.0e8, 6.0e8, 400_124_482_126_318_236.0),
    // Barely space-like: the two terms agree to three digits, so the answer is almost all `c`.
    (1.0, 1.0e8, 2.0e8, 2.0e8, 124_482_126_318_236.0),
    // Space-like at Δt = 2 s, which squares the light travel term to 4c².
    (2.0, 3.0e8, 4.0e8, 12.0e8, 1_330_497_928_505_272_944.0),
    // Time-like: the time term dominates by 64x and the sign turns over.
    (10.0, 1.0e8, 2.0e8, 3.0e8, -8_847_551_787_368_176_400.0),
    // Simultaneous events: the space term alone, exact in binary.
    (0.0, 1.0, 2.0, 3.0, 14.0),
];

fn lorentzian_pair(
    dt: FloatType,
    dx: FloatType,
    dy: FloatType,
    dz: FloatType,
) -> (
    LorentzianSpacetime<FloatType>,
    LorentzianSpacetime<FloatType>,
) {
    let a = LorentzianSpacetime::new(1, dx, dy, dz, dt, TimeScale::Second);
    let b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
    (a, b)
}

#[test]
fn test_interval_squared_matches_the_closed_form() {
    for (dt, dx, dy, dz, expected) in INTERVAL_TABLE {
        let (a, b) = lorentzian_pair(dt, dx, dy, dz);
        let got = a.interval_squared(&b);

        // Relative, because the two terms are near 1e17 and cancel: a few ULP of the operands is
        // the floor. The margin still resolves a `c` wrong by one metre per second, which shifts
        // the third row by five parts per million.
        let tolerance = expected.abs() * 1e-9;
        assert!(
            (got - expected).abs() <= tolerance,
            "Δt={dt}, Δ=({dx}, {dy}, {dz}): expected {expected}, got {got}"
        );
    }
}

#[test]
fn test_interval_squared_uses_all_three_spatial_axes() {
    // Simultaneous events, so the interval reduces to the squared spatial separation and the
    // closed form is a sum of three integer squares, exact in binary. Every row has three
    // distinct nonzero magnitudes, so an axis dropped or an axis read twice in place of another
    // lands somewhere else: on (3, 4, 12) a lost dz gives 25 against the true 169.
    let rows: [(FloatType, FloatType, FloatType, FloatType); 5] = [
        (3.0, 4.0, 12.0, 169.0),
        (1.0, 2.0, 4.0, 21.0),
        (5.0, 7.0, 11.0, 195.0),
        (2.0, 6.0, 9.0, 121.0),
        (-3.0, 4.0, -12.0, 169.0), // signs cancel under the squares
    ];

    for (dx, dy, dz, expected) in rows {
        let (a, b) = lorentzian_pair(0.0, dx, dy, dz);
        assert_eq!(a.interval_squared(&b), expected, "for ({dx}, {dy}, {dz})");
    }
}

#[test]
fn test_interval_squared_is_symmetric_in_its_operands() {
    // Only squared differences enter, so swapping the two events cannot change the answer.
    for (dt, dx, dy, dz, _) in INTERVAL_TABLE {
        let (a, b) = lorentzian_pair(dt, dx, dy, dz);
        assert_eq!(a.interval_squared(&b), b.interval_squared(&a));
    }
}

#[test]
fn test_interval_squared_is_independent_of_the_time_scale() {
    // The same physical separation, written in three units. `time()` converts to seconds before
    // the interval is formed, so all three must agree; an interval taken on the raw `t` field
    // would differ by the conversion factor squared.
    let space = (2.0e8, 3.0e8, 6.0e8);

    let in_seconds = {
        let a = LorentzianSpacetime::new(1, space.0, space.1, space.2, 60.0, TimeScale::Second);
        let b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
        a.interval_squared(&b)
    };
    let in_minutes = {
        let a = LorentzianSpacetime::new(1, space.0, space.1, space.2, 1.0, TimeScale::Minute);
        let b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Minute);
        a.interval_squared(&b)
    };
    let in_milliseconds = {
        let a = LorentzianSpacetime::new(
            1,
            space.0,
            space.1,
            space.2,
            60_000.0,
            TimeScale::Millisecond,
        );
        let b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Millisecond);
        a.interval_squared(&b)
    };

    assert_eq!(in_seconds, in_minutes);
    assert_eq!(in_seconds, in_milliseconds);

    // And the shared value is time-like: 60 s of light travel dwarfs 7e8 m of separation.
    assert!(in_seconds < 0.0);
}
