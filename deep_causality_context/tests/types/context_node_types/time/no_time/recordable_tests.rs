/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Recordable<TimeRecord>` for `NoTime`: writing is `Unrecordable`, reading any variant is
//! `WrongVariant`, both naming the node. Expected values are the identifiers passed in.
//!
//! Corner cases (rows A to K): C every variant of the record refused,
//! `test_reading_any_record_is_refused`; every other row n/a (the type holds no value).
use deep_causality_context::{NoTime, TimeScale};
use deep_causality_context_store::{ProjectionError, Recordable, TimeRecord};

#[test]
fn test_writing_is_unrecordable() {
    assert_eq!(
        NoTime::new().to_record(),
        Err(ProjectionError::Unrecordable(0, "NoTime"))
    );
}

#[test]
fn test_reading_any_record_is_refused() {
    let records = [
        TimeRecord::Newtonian {
            scale: TimeScale::Second,
            value: 1.0,
        },
        TimeRecord::Minkowski {
            scale: TimeScale::Second,
            value: 1.0,
        },
        TimeRecord::Discrete {
            scale: TimeScale::Steps,
            tick: 1,
        },
        TimeRecord::Entropic { tick: 1 },
    ];
    for record in records {
        let found = record.kind_name();
        assert_eq!(
            NoTime::from_record(8, record),
            Err(ProjectionError::WrongVariant(8, "NoTime", found))
        );
    }
}
