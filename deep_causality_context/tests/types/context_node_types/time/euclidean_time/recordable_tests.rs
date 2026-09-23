/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<TimeRecord>` for EuclideanTime. Expected records are the constructor literals under their
//! named fields; the wrong-variant refusals name the node.
//!
//! Corner cases (rows A to K): C every other `TimeRecord` variant refused,
//! `test_every_other_variant_is_refused`; F/G zero and a negative value,
//! `test_zero_and_negative_round_trip`; H the scale `Symbolic`, which the time kinds still
//! accept, in `test_round_trip`; I non-finite, `test_non_finite_round_trips`; K `Float106` narrows
//! and `BFloat16` widens, `test_precision_is_spent_at_the_bound`; every other row n/a.
use deep_causality_context::{EuclideanTime, TimeScale};
use deep_causality_context_store::{ProjectionError, Recordable, TimeRecord};
use deep_causality_num::{BFloat16, Float106};

#[test]
fn test_round_trip() {
    let node = EuclideanTime::new(3, TimeScale::Symbolic, 2.5);
    let record = node.to_record().unwrap();
    assert_eq!(
        record,
        TimeRecord::Euclidean {
            scale: TimeScale::Symbolic,
            value: 2.5
        }
    );
    assert_eq!(EuclideanTime::from_record(3, record), Ok(node));
}

#[test]
fn test_every_other_variant_is_refused() {
    let others: [(&str, TimeRecord); 3] = [
        (
            "Lorentzian",
            TimeRecord::Lorentzian {
                scale: TimeScale::Second,
                value: 1.0,
            },
        ),
        (
            "Discrete",
            TimeRecord::Discrete {
                scale: TimeScale::Steps,
                tick: 1,
            },
        ),
        ("Entropic", TimeRecord::Entropic { tick: 1 }),
    ];
    for (found, record) in others {
        assert_eq!(
            EuclideanTime::<f64>::from_record(9, record),
            Err(ProjectionError::WrongVariant(9, "Euclidean", found))
        );
    }
}

#[test]
fn test_zero_and_negative_round_trip() {
    for node in [
        EuclideanTime::new(1, TimeScale::NoScale, 0.0),
        EuclideanTime::new(1, TimeScale::Nanoseconds, -4.0),
    ] {
        assert_eq!(
            EuclideanTime::from_record(1, node.to_record().unwrap()),
            Ok(node)
        );
    }
}

#[test]
fn test_non_finite_round_trips() {
    let node = EuclideanTime::new(1, TimeScale::Second, f64::NAN);
    let restored = EuclideanTime::<f64>::from_record(1, node.to_record().unwrap()).unwrap();
    assert_ne!(restored, node);
    assert_eq!(restored.to_record().unwrap().kind_name(), "Euclidean");
}

#[test]
fn test_precision_is_spent_at_the_bound() {
    let low = 2f64.powi(-70);
    let wide = EuclideanTime::new(2, TimeScale::Second, Float106::new(1.0, low));
    let restored = EuclideanTime::<Float106>::from_record(2, wide.to_record().unwrap()).unwrap();
    assert_eq!(
        restored,
        EuclideanTime::new(2, TimeScale::Second, Float106::new(1.0, 0.0))
    );
    let narrow = EuclideanTime::new(2, TimeScale::Second, BFloat16::from(1.5));
    assert_eq!(
        EuclideanTime::<BFloat16>::from_record(2, narrow.to_record().unwrap()),
        Ok(narrow)
    );
}
