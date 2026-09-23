/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Storable` for `String`. Expected values are the literals written.
//!
//! Corner cases (rows A to K): A the empty string, `test_empty_and_unicode_round_trip`; every
//! other row n/a.

use deep_causality_context::Storable;
use deep_causality_context_store::{DataRecord, ProjectionError};

#[test]
fn test_a_string_is_text() {
    assert_eq!(
        "ok".to_string().to_record(),
        DataRecord::Text("ok".to_string())
    );
    assert_eq!(
        String::from_record(1, DataRecord::Text("ok".to_string())),
        Ok("ok".to_string())
    );
}

#[test]
fn test_empty_and_unicode_round_trip() {
    for text in ["", "ünïcødé ✓"] {
        let owned = text.to_string();
        assert_eq!(String::from_record(2, owned.to_record()), Ok(owned));
    }
}

#[test]
fn test_a_wrong_payload_names_the_node() {
    assert_eq!(
        String::from_record(3, DataRecord::Flag(true)),
        Err(ProjectionError::WrongPayload(3, "Text", "Flag"))
    );
}
