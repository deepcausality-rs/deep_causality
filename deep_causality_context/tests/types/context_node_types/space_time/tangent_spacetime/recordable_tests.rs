/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<SpaceTimeRecord>` for TangentSpacetime. Expected records are the constructor literals under
//! their named fields, all distinct so a swapped field is caught.
//!
//! Corner cases (rows A to K): C every other `SpaceTimeRecord` variant refused,
//! `test_every_other_variant_is_refused`; F/G zero and negative coordinates,
//! `test_zero_and_negative_round_trip`; I a non-finite coordinate round-trips,
//! `test_non_finite_round_trips`, and a non-finite metric entry is refused,
//! `test_a_record_whose_tensor_is_not_a_lorentzian_metric_is_refused`; J a finite value past the range of `f32` and `BFloat16` refused,
//! `test_a_value_past_the_scalar_range_is_refused`; K `Float106`
//! narrows and `BFloat16` widens, `test_precision_is_spent_at_the_bound`; every other row n/a.
//! The metric tensor is symmetric and Lorentzian, with every entry of its upper triangle distinct,
//! so a misplaced or default metric is caught: `test_the_stored_metric_is_restored_not_the_default`.
//! Its eigenvalues, from `numpy.linalg.eigvalsh`, are one negative and three positive.
use deep_causality_context::{MetricTensor4D, MetricTensorError, TangentSpacetime, TimeScale};
use deep_causality_context_store::{ProjectionError, Recordable, SpaceTimeRecord};
use deep_causality_num::{BFloat16, Float106};

fn metric() -> [[f64; 4]; 4] {
    [
        [-30.5, 0.5, 1.5, 2.5],
        [0.5, 10.5, 3.5, 4.5],
        [1.5, 3.5, 11.5, 5.5],
        [2.5, 4.5, 5.5, 12.5],
    ]
}

fn node() -> TangentSpacetime<f64> {
    let mut node = TangentSpacetime::new(3, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5);
    node.update_metric_tensor(metric())
        .expect("the fixture tensor is a Lorentzian metric");
    node
}

#[test]
fn test_round_trip() {
    let node = node();
    let record = node.to_record().unwrap();
    assert_eq!(
        record,
        SpaceTimeRecord::Tangent {
            x: 1.5,
            y: 2.5,
            z: 3.5,
            t: 4.5,
            dt: 5.5,
            dx: 6.5,
            dy: 7.5,
            dz: 8.5,
            metric: metric()
        }
    );
    assert_eq!(TangentSpacetime::from_record(3, record), Ok(node));
}

#[test]
fn test_the_stored_metric_is_restored_not_the_default() {
    let restored = TangentSpacetime::<f64>::from_record(3, node().to_record().unwrap()).unwrap();
    assert_eq!(restored.metric_tensor()[3][2], 5.5);
    assert_eq!(restored.metric_tensor()[0][3], 2.5);
    assert_eq!(restored.metric_tensor(), metric());
    let fresh = TangentSpacetime::new(3, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5);
    assert_ne!(restored.metric_tensor(), fresh.metric_tensor());
}

#[test]
fn test_every_other_variant_is_refused() {
    let others: [(&str, SpaceTimeRecord); 3] = [
        (
            "Galilean",
            SpaceTimeRecord::Galilean {
                t: 4.0,
                x: 1.0,
                y: 2.0,
                z: 3.0,
                scale: TimeScale::Second,
            },
        ),
        (
            "Newtonian",
            SpaceTimeRecord::Newtonian {
                t: 4.0,
                x: 1.0,
                y: 2.0,
                z: 3.0,
                scale: TimeScale::Second,
            },
        ),
        (
            "Minkowski",
            SpaceTimeRecord::Minkowski {
                t: 4.0,
                x: 1.0,
                y: 2.0,
                z: 3.0,
                scale: TimeScale::Second,
            },
        ),
    ];
    for (found, record) in others {
        assert_eq!(
            TangentSpacetime::<f64>::from_record(9, record),
            Err(ProjectionError::WrongVariant(9, "Tangent", found))
        );
    }
}

#[test]
fn test_zero_and_negative_round_trip() {
    for node in [
        TangentSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        TangentSpacetime::new(1, -1.0, -2.0, -3.0, -4.0, -1.0, -0.5, -0.25, -0.125),
    ] {
        assert_eq!(
            TangentSpacetime::from_record(1, node.to_record().unwrap()),
            Ok(node)
        );
    }
}

#[test]
fn test_non_finite_round_trips() {
    let node = TangentSpacetime::new(1, f64::INFINITY, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
    assert_eq!(
        TangentSpacetime::from_record(1, node.to_record().unwrap()),
        Ok(node)
    );
}

#[test]
fn test_precision_is_spent_at_the_bound() {
    let low = 2f64.powi(-70);
    let f = |v: f64| Float106::new(v, low);
    let wide = TangentSpacetime::new(
        2,
        f(1.0),
        f(2.0),
        f(3.0),
        f(4.0),
        f(1.0),
        f(0.5),
        f(0.25),
        f(0.125),
    );
    let restored = TangentSpacetime::<Float106>::from_record(2, wide.to_record().unwrap()).unwrap();
    let e = |v: f64| Float106::new(v, 0.0);
    let mut expected = TangentSpacetime::new(
        2,
        e(1.0),
        e(2.0),
        e(3.0),
        e(4.0),
        e(1.0),
        e(0.5),
        e(0.25),
        e(0.125),
    );
    // The default metric holds c² exactly in two halves; the record holds a double, so the
    // restored metric is the narrowed one.
    expected
        .update_metric_tensor(
            expected
                .metric_tensor()
                .map(|row| row.map(|v| e(f64::from(v)))),
        )
        .expect("the narrowed default metric is still Lorentzian");
    assert_eq!(restored.metric_tensor(), expected.metric_tensor());
    assert_eq!(restored, expected);
    let b = |v: f64| BFloat16::from(v);
    let narrow = TangentSpacetime::new(
        2,
        b(1.5),
        b(2.5),
        b(3.5),
        b(4.5),
        b(1.0),
        b(0.5),
        b(0.25),
        b(0.125),
    );
    assert_eq!(
        TangentSpacetime::<BFloat16>::from_record(2, narrow.to_record().unwrap()),
        Ok(narrow)
    );
}

#[test]
fn test_a_record_whose_tensor_is_not_a_lorentzian_metric_is_refused() {
    // A record can hold any sixteen numbers; the restore accepts only a finite symmetric tensor of
    // signature (−, +, +, +), the invariant every TangentSpacetime keeps. The refusal carries the
    // message `update_metric_tensor` gives for the same tensor, so each broken rule reads as its
    // own: a non-finite entry, the asymmetric pair, the eigenvalue count.
    let record = |metric: [[f64; 4]; 4]| SpaceTimeRecord::Tangent {
        t: 0.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        dt: 1.0,
        dx: 0.0,
        dy: 0.0,
        dz: 0.0,
        metric,
    };
    let mut non_finite = metric();
    non_finite[2][3] = f64::NAN;
    let mut asymmetric = metric();
    asymmetric[1][3] = 9.0;
    let riemannian = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let cases = [
        (non_finite, MetricTensorError::NonFinite(2, 3)),
        (asymmetric, MetricTensorError::Asymmetric(1, 3)),
        (riemannian, MetricTensorError::Signature(4, 0, 0)),
    ];
    for (bad, expected) in cases {
        let refused = TangentSpacetime::<f64>::new(6, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0)
            .update_metric_tensor(bad);
        assert_eq!(refused, Err(expected));
        assert_eq!(
            TangentSpacetime::<f64>::from_record(6, record(bad)),
            Err(ProjectionError::Rejected(6, expected.to_string()))
        );
    }
}

#[test]
fn test_a_value_past_the_scalar_range_is_refused() {
    // Row J. 1e300 is a finite f64 past the range of f32 and BFloat16, which would hold it as an
    // infinity; the restore refuses it and names the value.
    let record = SpaceTimeRecord::Tangent {
        t: 0.0,
        x: 0.0,
        y: 0.0,
        z: 0.0,
        dt: 1.0,
        dx: 0.0,
        dy: 0.0,
        dz: 0.0,
        metric: [
            [-1.0, 0.0, 0.0, 0.0],
            [0.0, 1e300, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };
    assert_eq!(
        TangentSpacetime::<f32>::from_record(4, record),
        Err(ProjectionError::Scalar(4, 1e300))
    );
    assert_eq!(
        TangentSpacetime::<BFloat16>::from_record(4, record),
        Err(ProjectionError::Scalar(4, 1e300))
    );
}
