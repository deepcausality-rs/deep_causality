/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `Storable` for `Vec<T>` and `Option<T>`. Expected values are the literals written; a hundred
//! values are `i as f64 / 4.0`, distinct and in order, so a dropped, duplicated or reordered
//! element is caught.
//!
//! Corner cases (rows A to K): A the empty vector and `None`, `test_empty_and_none`; B one
//! element and `Some`, `test_one_element_and_some`; D a list of two read as an `Option`,
//! refused, `test_a_longer_list_is_not_an_option`; every other row n/a.

use deep_causality_context::Storable;
use deep_causality_context_store::{DataRecord, ProjectionError};

#[test]
fn test_a_hundred_values_come_back_in_order() {
    let series: Vec<f64> = (0..100).map(|i| f64::from(i) / 4.0).collect();
    let record = series.to_record();
    let DataRecord::List(items) = &record else {
        panic!("a List record");
    };
    assert_eq!(items.len(), 100);
    assert_eq!(items[3], DataRecord::Number(0.75));
    assert_eq!(Vec::<f64>::from_record(1, record), Ok(series));
}

#[test]
fn test_empty_and_none() {
    assert_eq!(Vec::<u64>::new().to_record(), DataRecord::List(vec![]));
    assert_eq!(
        Vec::<u64>::from_record(1, DataRecord::List(vec![])),
        Ok(vec![])
    );
    assert_eq!(None::<u64>.to_record(), DataRecord::List(vec![]));
    assert_eq!(
        Option::<u64>::from_record(1, DataRecord::List(vec![])),
        Ok(None)
    );
}

#[test]
fn test_one_element_and_some() {
    assert_eq!(
        vec![true].to_record(),
        DataRecord::List(vec![DataRecord::Flag(true)])
    );
    assert_eq!(
        Some(3u64).to_record(),
        DataRecord::List(vec![DataRecord::Count(3)])
    );
    assert_eq!(
        Option::<u64>::from_record(1, DataRecord::List(vec![DataRecord::Count(3)])),
        Ok(Some(3))
    );
}

#[test]
fn test_a_longer_list_is_not_an_option() {
    let two = DataRecord::List(vec![DataRecord::Count(1), DataRecord::Count(2)]);
    assert_eq!(
        Option::<u64>::from_record(5, two),
        Err(ProjectionError::WrongPayload(5, "Option", "List"))
    );
}

#[test]
fn test_wrong_payloads_name_the_node() {
    assert_eq!(
        Vec::<u64>::from_record(6, DataRecord::Count(1)),
        Err(ProjectionError::WrongPayload(6, "List", "Count"))
    );
    assert_eq!(
        Option::<u64>::from_record(6, DataRecord::Count(1)),
        Err(ProjectionError::WrongPayload(6, "List", "Count"))
    );
    // An element of the wrong arm is refused by the element's own reader.
    assert_eq!(
        Vec::<u64>::from_record(7, DataRecord::List(vec![DataRecord::Flag(true)])),
        Err(ProjectionError::WrongPayload(7, "Count", "Flag"))
    );
}

#[test]
fn test_nested_collections() {
    let nested = vec![vec![1u64, 2], vec![], vec![3]];
    assert_eq!(
        Vec::<Vec<u64>>::from_record(1, nested.to_record()),
        Ok(nested)
    );
    let optional_list = Some(vec![String::from("a")]);
    assert_eq!(
        Option::<Vec<String>>::from_record(1, optional_list.to_record()),
        Ok(optional_list)
    );
}
