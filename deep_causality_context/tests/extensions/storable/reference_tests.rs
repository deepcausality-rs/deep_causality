/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Storable` for `SubstrateRef`. Expected values are the literals written.
//!
//! Corner cases (rows A to K): A the default reference with two empty halves,
//! `test_a_default_reference_round_trips`; every other row n/a.

use deep_causality_context::{Storable, SubstrateRef};
use deep_causality_context_store::{DataRecord, ProjectionError};

#[test]
fn test_a_reference_is_a_reference() {
    let s = SubstrateRef::new("series".to_string(), "k".to_string());
    let t = DataRecord::Reference(SubstrateRef::new("series".to_string(), "k".to_string()));
    assert_eq!(s.to_record(), t);
    assert_eq!(SubstrateRef::from_record(1, t.clone()), Ok(s.clone()));
    // S -> T -> S and T -> S -> T.
    assert_eq!(SubstrateRef::from_record(1, s.to_record()), Ok(s));
    assert_eq!(
        SubstrateRef::from_record(1, t.clone()).map(|r| r.to_record()),
        Ok(t)
    );
}

#[test]
fn test_a_default_reference_round_trips() {
    let reference = SubstrateRef::default();
    assert_eq!(
        SubstrateRef::from_record(1, reference.to_record()),
        Ok(reference)
    );
}

#[test]
fn test_a_wrong_payload_names_the_node() {
    assert_eq!(
        SubstrateRef::from_record(3, DataRecord::Number(1.0)),
        Err(ProjectionError::WrongPayload(3, "Reference", "Number"))
    );
}
