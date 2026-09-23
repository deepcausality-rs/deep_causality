/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<TimeRecord>` for EntropicTime, a tick-based time with no scalar parameter. Expected
//! records are the constructor literals.
//!
//! Corner cases (rows A to K): C every other `TimeRecord` variant refused,
//! `test_every_other_variant_is_refused`; F tick 0 and H tick `u64::MAX`,
//! `test_zero_and_the_top_tick_round_trip`; every other row n/a (a tick has no sign, no
//! non-finite value and no precision).
use deep_causality_context::{EntropicTime, TimeScale};
use deep_causality_context_store::{ProjectionError, Recordable, TimeRecord};

#[test]
fn test_round_trip() {
    let node = EntropicTime::new(3, 42);
    let record = node.to_record().unwrap();
    assert_eq!(record, TimeRecord::Entropic { tick: 42 });
    assert_eq!(EntropicTime::from_record(3, record), Ok(node));
}

#[test]
fn test_every_other_variant_is_refused() {
    let others: [(&str, TimeRecord); 3] = [
        (
            "Euclidean",
            TimeRecord::Euclidean {
                scale: TimeScale::Second,
                value: 1.0,
            },
        ),
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
    ];
    for (found, record) in others {
        assert_eq!(
            EntropicTime::from_record(9, record),
            Err(ProjectionError::WrongVariant(9, "Entropic", found))
        );
    }
}

#[test]
fn test_zero_and_the_top_tick_round_trip() {
    for node in [EntropicTime::new(1, 0), EntropicTime::new(1, u64::MAX)] {
        assert_eq!(
            EntropicTime::from_record(1, node.to_record().unwrap()),
            Ok(node)
        );
    }
}
