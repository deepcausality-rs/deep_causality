/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructors, read back by pattern; no
//! expectation is computed.
//!
//! Corner cases (rows A to K): A/B n/a (a record is one value); C coinciding values in
//! `test_equal_fields_under_different_variants` (same numbers, two variants, not equal); D/E n/a;
//! F zero tick and G negative value in `test_zero_and_negative`; H `u64::MAX` tick in the same
//! test; I non-finite value in `test_non_finite_value`; J/K n/a.

use deep_causality_context_store::{TimeRecord, TimeScale};

fn one_of_each() -> [TimeRecord; 4] {
    [
        TimeRecord::Euclidean {
            scale: TimeScale::Second,
            value: 1.5,
        },
        TimeRecord::Lorentzian {
            scale: TimeScale::Millisecond,
            value: 2.5,
        },
        TimeRecord::Discrete {
            scale: TimeScale::Steps,
            tick: 3,
        },
        TimeRecord::Entropic { tick: 4 },
    ]
}

#[test]
fn test_four_variants_with_distinct_names() {
    let names: Vec<&str> = one_of_each().iter().map(TimeRecord::kind_name).collect();
    assert_eq!(names, ["Euclidean", "Lorentzian", "Discrete", "Entropic"]);
}

#[test]
fn test_fields_are_named_and_the_record_is_copy() {
    let record = TimeRecord::Discrete {
        scale: TimeScale::Steps,
        tick: 3,
    };
    let copy = record;
    assert_eq!(record, copy);
    let TimeRecord::Discrete { scale, tick } = copy else {
        panic!("a Discrete record");
    };
    assert_eq!(scale, TimeScale::Steps);
    assert_eq!(tick, 3);
    assert!(format!("{record:?}").contains("Discrete"));
}

#[test]
fn test_equal_fields_under_different_variants() {
    let euclidean = TimeRecord::Euclidean {
        scale: TimeScale::Second,
        value: 1.0,
    };
    let lorentzian = TimeRecord::Lorentzian {
        scale: TimeScale::Second,
        value: 1.0,
    };
    assert_ne!(euclidean, lorentzian);
    assert_ne!(
        TimeRecord::Discrete {
            scale: TimeScale::NoScale,
            tick: 0
        },
        TimeRecord::Entropic { tick: 0 }
    );
}

#[test]
fn test_zero_and_negative() {
    let zero = TimeRecord::Entropic { tick: 0 };
    let max = TimeRecord::Entropic { tick: u64::MAX };
    assert_ne!(zero, max);
    let negative = TimeRecord::Lorentzian {
        scale: TimeScale::Second,
        value: -2.0,
    };
    let TimeRecord::Lorentzian { value, .. } = negative else {
        panic!("a Lorentzian record");
    };
    assert_eq!(value, -2.0);
}

#[test]
fn test_non_finite_value() {
    let inf = TimeRecord::Euclidean {
        scale: TimeScale::Second,
        value: f64::INFINITY,
    };
    assert_eq!(inf, inf);
    let nan = TimeRecord::Euclidean {
        scale: TimeScale::Second,
        value: f64::NAN,
    };
    assert_ne!(nan, nan);
}
