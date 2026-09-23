/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<TimeRecord>` for `TimeKind`: total, four variants, four arms. Expected records
//! are the inner constructors' literals.
//!
//! Corner cases (rows A to K): C every variant maps to a distinct record variant,
//! `test_every_variant_round_trips`; every other row is the inner types' and is in their files.
use deep_causality_context::{
    DiscreteTime, EntropicTime, EuclideanTime, LorentzianTime, TimeKind, TimeScale,
};
use deep_causality_context_store::{Recordable, TimeRecord};

#[test]
fn test_every_variant_round_trips() {
    let kinds: [TimeKind<f64>; 4] = [
        TimeKind::Euclidean(EuclideanTime::new(1, TimeScale::Second, 1.5)),
        TimeKind::Entropic(EntropicTime::new(2, 7)),
        TimeKind::Discrete(DiscreteTime::new(3, TimeScale::Steps, 9)),
        TimeKind::Lorentzian(LorentzianTime::new(4, TimeScale::Nanoseconds, 2.5)),
    ];
    let mut names = Vec::new();
    for (id, kind) in (1u64..).zip(kinds) {
        let record = kind.to_record().unwrap();
        names.push(record.kind_name());
        assert_eq!(TimeKind::from_record(id, record), Ok(kind));
    }
    assert_eq!(names, ["Euclidean", "Entropic", "Discrete", "Lorentzian"]);
}

#[test]
fn test_the_record_lands_in_the_matching_variant() {
    let restored = TimeKind::<f64>::from_record(5, TimeRecord::Entropic { tick: 3 }).unwrap();
    assert_eq!(restored, TimeKind::Entropic(EntropicTime::new(5, 3)));
}
