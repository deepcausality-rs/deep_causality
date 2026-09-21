/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_context::FloatType;
use deep_causality_context::*;

#[test]
fn test_space_temporal_interval_time_is_raw_seconds() {
    // TangentSpacetime has no time_scale; `time()` returns `t` unchanged.
    let s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 42.0, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(SpaceTemporalInterval::time(&s), 42.0);
}

#[test]
fn test_space_temporal_interval_position() {
    let s = TangentSpacetime::new(1, 1.0, 2.0, 3.0, 4.0, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(SpaceTemporalInterval::position(&s), [1.0, 2.0, 3.0]);
}

// =============================================================================
// interval_squared() — the full 4x4 double contraction
// =============================================================================

/// Builds the pair whose coordinate difference `self - other` is `v = [Δt, Δx, Δy, Δz]`,
/// with `other` at the origin of both space and time. The tangent components play no part in
/// the interval, so they are held at zero.
fn tangent_pair_with_difference(
    v: [FloatType; 4],
    metric: [[FloatType; 4]; 4],
) -> (TangentSpacetime<FloatType>, TangentSpacetime<FloatType>) {
    let [dt, dx, dy, dz] = v;
    let mut a = TangentSpacetime::new(1, dx, dy, dz, dt, 0.0, 0.0, 0.0, 0.0);
    a.update_metric_tensor(metric);
    let b = TangentSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    (a, b)
}

/// Contractions `Σ g[u][w]·v[u]·v[w]` evaluated by hand, each term an integer.
///
/// Every tensor is asymmetric and has off-diagonal entries in both triangles, and every `v` has
/// four distinct nonzero components. A diagonal fixture cannot separate these: on the first row
/// a diagonal-only sum gives 30 and a loop bound of 3 gives 36, against the true 120.
///
/// One defect a quadratic form cannot see, whatever the tensor: `vᵀGv` equals `vᵀGᵀv` for every
/// `G`, so reading `g[w][u]` in place of `g[u][w]` is not an error here and no fixture can make
/// it one.
const CONTRACTION_TABLE: [([[FloatType; 4]; 4], [FloatType; 4], FloatType); 3] = [
    (
        [
            [1.0, 2.0, 0.0, 0.0],
            [0.0, 1.0, 3.0, 0.0],
            [0.0, 0.0, 1.0, 4.0],
            [5.0, 0.0, 0.0, 1.0],
        ],
        [1.0, 2.0, 3.0, 4.0],
        120.0,
    ),
    (
        [
            [-2.0, 1.0, 0.0, 3.0],
            [0.0, 4.0, -1.0, 0.0],
            [2.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 5.0, -3.0],
        ],
        [2.0, -1.0, 3.0, 5.0],
        48.0,
    ),
    (
        // Every entry nonzero and distinct, so no index pair is masked by a zero.
        [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ],
        [1.0, 3.0, -2.0, 5.0],
        539.0,
    ),
];

#[test]
fn test_interval_squared_contracts_the_full_metric_tensor() {
    for (metric, v, expected) in CONTRACTION_TABLE {
        let (a, b) = tangent_pair_with_difference(v, metric);
        assert_eq!(
            a.interval_squared(&b),
            expected,
            "metric {metric:?} against v {v:?}"
        );
    }
}

#[test]
fn test_interval_squared_orders_the_difference_vector_as_t_x_y_z() {
    // The tensor's rows differ, so the contraction changes if `v` is assembled in another order.
    // Under the third table row, `[Δx, Δy, Δz, Δt]` would give a different number.
    let metric = CONTRACTION_TABLE[2].0;

    let (a, b) = tangent_pair_with_difference([1.0, 0.0, 0.0, 0.0], metric);
    assert_eq!(
        a.interval_squared(&b),
        metric[0][0],
        "Δt alone picks g[0][0]"
    );

    let (a, b) = tangent_pair_with_difference([0.0, 1.0, 0.0, 0.0], metric);
    assert_eq!(
        a.interval_squared(&b),
        metric[1][1],
        "Δx alone picks g[1][1]"
    );

    let (a, b) = tangent_pair_with_difference([0.0, 0.0, 1.0, 0.0], metric);
    assert_eq!(
        a.interval_squared(&b),
        metric[2][2],
        "Δy alone picks g[2][2]"
    );

    let (a, b) = tangent_pair_with_difference([0.0, 0.0, 0.0, 1.0], metric);
    assert_eq!(
        a.interval_squared(&b),
        metric[3][3],
        "Δz alone picks g[3][3]"
    );
}

#[test]
fn test_interval_squared_is_quadratic_in_the_separation() {
    // A double contraction is homogeneous of degree two, so scaling the separation by k scales
    // the interval by k². A term that entered linearly would break this.
    let (metric, v, expected) = CONTRACTION_TABLE[0];

    for k in [2.0 as FloatType, -3.0, 0.5] {
        let scaled = [v[0] * k, v[1] * k, v[2] * k, v[3] * k];
        let (a, b) = tangent_pair_with_difference(scaled, metric);
        assert_eq!(a.interval_squared(&b), expected * k * k, "scale {k}");
    }
}

#[test]
fn test_interval_squared_of_coincident_events_is_zero_under_any_metric() {
    for (metric, _, _) in CONTRACTION_TABLE {
        let (a, b) = tangent_pair_with_difference([0.0, 0.0, 0.0, 0.0], metric);
        assert_eq!(a.interval_squared(&b), 0.0);
    }
}

#[test]
fn test_default_metric_reproduces_the_flat_minkowski_interval() {
    // `TangentSpacetime::new` installs g = diag(-c², 1, 1, 1), so a tangent pair under its own
    // default tensor must report the same interval as a Lorentzian pair, which reaches the
    // answer through the trait's default `-(c·Δt)² + Δx² + Δy² + Δz²` and shares no code with
    // the contraction. The identity pins the speed of light inside the constructor.
    let rows: [(FloatType, FloatType, FloatType, FloatType); 3] = [
        (1.0, 2.0e8, 3.0e8, 6.0e8),
        (10.0, 1.0e8, 2.0e8, 3.0e8),
        (0.0, 3.0, 4.0, 12.0),
    ];

    for (dt, dx, dy, dz) in rows {
        let tangent_a = TangentSpacetime::new(1, dx, dy, dz, dt, 0.0, 0.0, 0.0, 0.0);
        let tangent_b = TangentSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let tangent = tangent_a.interval_squared(&tangent_b);

        let lorentzian_a = LorentzianSpacetime::new(1, dx, dy, dz, dt, TimeScale::Second);
        let lorentzian_b = LorentzianSpacetime::new(2, 0.0, 0.0, 0.0, 0.0, TimeScale::Second);
        let lorentzian = lorentzian_a.interval_squared(&lorentzian_b);

        let tolerance = lorentzian.abs() * 1e-12;
        assert!(
            (tangent - lorentzian).abs() <= tolerance,
            "Δt={dt}, Δ=({dx}, {dy}, {dz}): tangent {tangent}, lorentzian {lorentzian}"
        );
    }
}
