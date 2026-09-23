/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<SpaceTimeRecord>` for LorentzianSpacetime. Expected records are the constructor literals under
//! their named fields, all distinct so a swapped field is caught.
//!
//! Corner cases (rows A to K): C every other `SpaceTimeRecord` variant refused,
//! `test_every_other_variant_is_refused`; F/G zero and negative coordinates,
//! `test_zero_and_negative_round_trip`; I non-finite, `test_non_finite_round_trips`; K `Float106`
//! narrows and `BFloat16` widens, `test_precision_is_spent_at_the_bound`; every other row n/a.
use deep_causality_context::{LorentzianSpacetime, TimeScale};
use deep_causality_context_store::{ProjectionError, Recordable, SpaceTimeRecord};
use deep_causality_num::{BFloat16, Float106};

#[test]
fn test_round_trip() {
    let node = LorentzianSpacetime::new(3, 1.5, 2.5, 3.5, 4.5, TimeScale::Millisecond);
    let record = node.to_record().unwrap();
    assert_eq!(
        record,
        SpaceTimeRecord::Lorentzian {
            x: 1.5,
            y: 2.5,
            z: 3.5,
            t: 4.5,
            scale: TimeScale::Millisecond
        }
    );
    assert_eq!(LorentzianSpacetime::from_record(3, record), Ok(node));
}

#[test]
fn test_every_other_variant_is_refused() {
    let others: [(&str, SpaceTimeRecord); 2] = [
        (
            "Euclidean",
            SpaceTimeRecord::Euclidean {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                t: 4.0,
                scale: TimeScale::Second,
            },
        ),
        (
            "Tangent",
            SpaceTimeRecord::Tangent {
                x: 1.0,
                y: 2.0,
                z: 3.0,
                t: 4.0,
                dt: 1.0,
                dx: 0.0,
                dy: 0.0,
                dz: 0.0,
                metric: [[0.0; 4]; 4],
            },
        ),
    ];
    for (found, record) in others {
        assert_eq!(
            LorentzianSpacetime::<f64>::from_record(9, record),
            Err(ProjectionError::WrongVariant(9, "Lorentzian", found))
        );
    }
}

#[test]
fn test_zero_and_negative_round_trip() {
    for node in [
        LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, 0.0, TimeScale::NoScale),
        LorentzianSpacetime::new(1, -1.0, -2.0, -3.0, -4.0, TimeScale::Year),
    ] {
        assert_eq!(
            LorentzianSpacetime::from_record(1, node.to_record().unwrap()),
            Ok(node)
        );
    }
}

#[test]
fn test_non_finite_round_trips() {
    let node = LorentzianSpacetime::new(1, 0.0, 0.0, 0.0, f64::INFINITY, TimeScale::Second);
    assert_eq!(
        LorentzianSpacetime::from_record(1, node.to_record().unwrap()),
        Ok(node)
    );
}

#[test]
fn test_precision_is_spent_at_the_bound() {
    let low = 2f64.powi(-70);
    let f = |v: f64| Float106::new(v, low);
    let wide = LorentzianSpacetime::new(2, f(1.0), f(2.0), f(3.0), f(4.0), TimeScale::Second);
    let restored =
        LorentzianSpacetime::<Float106>::from_record(2, wide.to_record().unwrap()).unwrap();
    let e = |v: f64| Float106::new(v, 0.0);
    assert_eq!(
        restored,
        LorentzianSpacetime::new(2, e(1.0), e(2.0), e(3.0), e(4.0), TimeScale::Second)
    );
    let b = |v: f64| BFloat16::from(v);
    let narrow = LorentzianSpacetime::new(2, b(1.5), b(2.5), b(3.5), b(4.5), TimeScale::Second);
    assert_eq!(
        LorentzianSpacetime::<BFloat16>::from_record(2, narrow.to_record().unwrap()),
        Ok(narrow)
    );
}
