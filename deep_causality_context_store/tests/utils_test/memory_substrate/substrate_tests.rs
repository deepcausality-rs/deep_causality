/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! `deposit` and `resolve` on the in-memory substrate, with both refusals. Expected values are the
//! records deposited; the reference's source is the constant the substrate declares and its key
//! is the node's identifier written out.
//!
//! Corner cases (rows A to K): A an empty `Fields` record in `test_a_value_round_trips`; F node
//! 0 in `test_a_reference_is_not_a_value`; H `u64::MAX` as a node in
//! `test_the_key_is_the_node_identifier`; every other row n/a.

use deep_causality_context_store::utils_test::{MemorySubstrate, block_on};
use deep_causality_context_store::{DataRecord, MemorySubstrateError, Substrate, SubstrateRef};

#[test]
fn test_a_value_round_trips() {
    let substrate = MemorySubstrate::new();
    let reading = DataRecord::Fields(vec![
        ("temperature".to_string(), DataRecord::Number(21.5)),
        ("samples".to_string(), DataRecord::Count(3)),
        ("status".to_string(), DataRecord::Text("ok".to_string())),
    ]);
    let reference = block_on(substrate.deposit(7, &reading)).unwrap();
    assert_eq!(reference.source(), "memory");
    assert_eq!(block_on(substrate.resolve(&reference)), Ok(reading));
    let empty = block_on(substrate.deposit(8, &DataRecord::Fields(vec![]))).unwrap();
    assert_eq!(
        block_on(substrate.resolve(&empty)),
        Ok(DataRecord::Fields(vec![]))
    );
}

#[test]
fn test_the_key_is_the_node_identifier() {
    let substrate = MemorySubstrate::new();
    let reference = block_on(substrate.deposit(u64::MAX, &DataRecord::Flag(true))).unwrap();
    assert_eq!(reference.key(), u64::MAX.to_string());
    assert_eq!(
        reference,
        SubstrateRef::new("memory".to_string(), u64::MAX.to_string())
    );
}

#[test]
fn test_a_reference_is_not_a_value() {
    let substrate = MemorySubstrate::new();
    let inner = SubstrateRef::new("s".to_string(), "k".to_string());
    assert_eq!(
        block_on(substrate.deposit(0, &DataRecord::Reference(inner))),
        Err(MemorySubstrateError::ReferenceRefused(0))
    );
    assert!(substrate.is_empty());
}

#[test]
fn test_an_unknown_reference_is_refused() {
    let substrate = MemorySubstrate::new();
    block_on(substrate.deposit(1, &DataRecord::Number(1.0))).unwrap();
    let wrong_key = SubstrateRef::new("memory".to_string(), "2".to_string());
    assert_eq!(
        block_on(substrate.resolve(&wrong_key)),
        Err(MemorySubstrateError::UnknownReference(wrong_key.clone()))
    );
    let wrong_source = SubstrateRef::new("file".to_string(), "1".to_string());
    assert_eq!(
        block_on(substrate.resolve(&wrong_source)),
        Err(MemorySubstrateError::UnknownReference(wrong_source))
    );
}
