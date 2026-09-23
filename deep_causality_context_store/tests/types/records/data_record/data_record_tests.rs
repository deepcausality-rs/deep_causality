/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Expected values are the literals handed to the constructors, read back through the public
//! API; no expectation is computed.
//!
//! Corner cases (rows A to K of the TDD protocol): A empty `List`/`Fields` in
//! `test_empty_list_and_fields`; B single-entry `Fields` in `test_eight_variants_with_distinct_names`;
//! C coinciding values in `test_fields_equality_is_ordered` (same entries, two orders); D index
//! degeneracy n/a (no index arithmetic); E thresholds n/a; F zero and G negative in
//! `test_zero_and_negative_scalars`; H exact boundary `u64::MAX`, `i64::MIN` in the same test;
//! I non-finite `f64::NAN` never equals itself, `test_nan_breaks_equality`; J overflow n/a; K
//! precision n/a (the record is `f64` by design).

use deep_causality_context_store::{DataRecord, SubstrateRef};

fn one_of_each() -> Vec<DataRecord> {
    vec![
        DataRecord::Number(1.5),
        DataRecord::Count(2),
        DataRecord::Integer(-3),
        DataRecord::Flag(true),
        DataRecord::Text("t".to_string()),
        DataRecord::Reference(SubstrateRef::new("s".to_string(), "k".to_string())),
        DataRecord::List(vec![DataRecord::Number(0.5)]),
        DataRecord::Fields(vec![("f".to_string(), DataRecord::Count(1))]),
    ]
}

#[test]
fn test_eight_variants_with_distinct_names() {
    let all = one_of_each();
    assert_eq!(all.len(), 8);
    let names: Vec<&str> = all.iter().map(DataRecord::kind_name).collect();
    assert_eq!(
        names,
        [
            "Number",
            "Count",
            "Integer",
            "Flag",
            "Text",
            "Reference",
            "List",
            "Fields"
        ]
    );
}

#[test]
fn test_a_struct_payload_is_representable() {
    let reading = DataRecord::Fields(vec![
        ("temperature".to_string(), DataRecord::Number(21.5)),
        ("samples".to_string(), DataRecord::Count(3)),
        ("status".to_string(), DataRecord::Text("ok".to_string())),
        (
            "history".to_string(),
            DataRecord::List(vec![DataRecord::Number(20.0), DataRecord::Number(21.0)]),
        ),
    ]);
    let DataRecord::Fields(entries) = &reading else {
        panic!("a Fields record");
    };
    assert_eq!(entries[0].0, "temperature");
    assert_eq!(entries[0].1, DataRecord::Number(21.5));
    assert_eq!(entries[1].1, DataRecord::Count(3));
    assert_eq!(entries[2].1, DataRecord::Text("ok".to_string()));
    assert_eq!(entries[3].1.kind_name(), "List");
}

#[test]
fn test_fields_equality_is_ordered() {
    let a = DataRecord::Fields(vec![
        ("x".to_string(), DataRecord::Number(1.0)),
        ("y".to_string(), DataRecord::Number(2.0)),
    ]);
    let same = DataRecord::Fields(vec![
        ("x".to_string(), DataRecord::Number(1.0)),
        ("y".to_string(), DataRecord::Number(2.0)),
    ]);
    let reordered = DataRecord::Fields(vec![
        ("y".to_string(), DataRecord::Number(2.0)),
        ("x".to_string(), DataRecord::Number(1.0)),
    ]);
    assert_eq!(a, same);
    assert_ne!(a, reordered);
}

#[test]
fn test_empty_list_and_fields() {
    let list = DataRecord::List(vec![]);
    let fields = DataRecord::Fields(vec![]);
    assert_eq!(list.kind_name(), "List");
    assert_eq!(fields.kind_name(), "Fields");
    assert_ne!(list, fields);
    assert_eq!(list, DataRecord::List(vec![]));
}

#[test]
fn test_zero_and_negative_scalars() {
    assert_eq!(DataRecord::Number(0.0), DataRecord::Number(0.0));
    assert_ne!(DataRecord::Number(0.0), DataRecord::Number(-1.0));
    assert_eq!(DataRecord::Count(u64::MAX), DataRecord::Count(u64::MAX));
    assert_eq!(DataRecord::Integer(i64::MIN), DataRecord::Integer(i64::MIN));
    assert_ne!(DataRecord::Integer(0), DataRecord::Count(0));
}

#[test]
fn test_nan_breaks_equality() {
    let nan = DataRecord::Number(f64::NAN);
    assert_ne!(nan, nan.clone());
    assert_eq!(nan.kind_name(), "Number");
}

#[test]
fn test_clone_and_debug() {
    for record in one_of_each() {
        let copy = record.clone();
        assert_eq!(record, copy);
        assert!(format!("{record:?}").contains(record.kind_name()));
    }
}
