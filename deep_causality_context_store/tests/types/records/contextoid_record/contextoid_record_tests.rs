/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructor; no expectation is computed.
//!
//! Corner cases (rows A to K): A/B n/a; C same identifier with a different payload and the same
//! payload with a different identifier, both in `test_clone_equality_and_debug`; D/E n/a; F
//! identifier 0 in `test_zero_identifier`; G n/a; H `u64::MAX` in the same test; I/J/K n/a.

use deep_causality_context_store::{ContextoidRecord, DataRecord, NodeRecord};

#[test]
fn test_new_and_getters() {
    let record = ContextoidRecord::new(3, NodeRecord::Data(DataRecord::Number(0.5)));
    assert_eq!(record.id(), 3);
    assert_eq!(record.node(), &NodeRecord::Data(DataRecord::Number(0.5)));
}

#[test]
fn test_zero_identifier() {
    assert_eq!(ContextoidRecord::new(0, NodeRecord::Root).id(), 0);
    assert_eq!(
        ContextoidRecord::new(u64::MAX, NodeRecord::Root).id(),
        u64::MAX
    );
}

#[test]
fn test_clone_equality_and_debug() {
    let record = ContextoidRecord::new(3, NodeRecord::Root);
    assert_eq!(record.clone(), record);
    assert_ne!(record, ContextoidRecord::new(4, NodeRecord::Root));
    assert_ne!(
        record,
        ContextoidRecord::new(3, NodeRecord::Data(DataRecord::Flag(true)))
    );
    assert!(format!("{record:?}").contains("Root"));
}
