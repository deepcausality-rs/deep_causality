/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! A one-byte node type pins the trait's two directions and its two refusals. Expected values are
//! the literals the probe is built from.
//!
//! Corner cases (rows A to K): H the sentinel byte at the top of the range, `u8::MAX`, in
//! `test_to_record_may_refuse` and `test_from_record_may_refuse_and_names_the_node`; F zero level
//! in `test_zero_level_round_trips`; every other row n/a.
use deep_causality_context_store::{ContextoidId, ProjectionError, Recordable};

/// A node type whose record is one byte. The byte `u8::MAX` is the level it cannot hold.
#[derive(Debug, PartialEq)]
struct Probe {
    id: ContextoidId,
    level: u8,
}

impl Recordable<u8> for Probe {
    fn to_record(&self) -> Result<u8, ProjectionError> {
        if self.level == u8::MAX {
            Err(ProjectionError::Unrecordable(self.id, "Probe"))
        } else {
            Ok(self.level)
        }
    }

    fn from_record(id: ContextoidId, record: u8) -> Result<Self, ProjectionError> {
        if record == u8::MAX {
            Err(ProjectionError::WrongVariant(id, "level", "sentinel"))
        } else {
            Ok(Self { id, level: record })
        }
    }
}

fn round_trip<T: Recordable<u8>>(id: ContextoidId, value: &T) -> Result<T, ProjectionError> {
    T::from_record(id, value.to_record()?)
}

#[test]
fn test_round_trip_through_the_generic_bound() {
    let probe = Probe { id: 3, level: 42 };
    assert_eq!(round_trip(3, &probe).unwrap(), probe);
}

#[test]
fn test_to_record_may_refuse() {
    let probe = Probe {
        id: 3,
        level: u8::MAX,
    };
    assert_eq!(
        probe.to_record(),
        Err(ProjectionError::Unrecordable(3, "Probe"))
    );
}

#[test]
fn test_from_record_may_refuse_and_names_the_node() {
    assert_eq!(
        Probe::from_record(8, u8::MAX),
        Err(ProjectionError::WrongVariant(8, "level", "sentinel"))
    );
}

#[test]
fn test_zero_level_round_trips() {
    let probe = Probe { id: 0, level: 0 };
    assert_eq!(round_trip(0, &probe).unwrap(), probe);
}
