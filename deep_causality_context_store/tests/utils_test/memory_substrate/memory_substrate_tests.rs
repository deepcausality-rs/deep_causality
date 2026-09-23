/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The substrate handle: construction, `Default`, and its count. Expected values are the records
//! deposited and the number of deposits.
//!
//! Corner cases (rows A to K): A an empty substrate in `test_a_new_substrate_is_empty`; C two
//! deposits under one node replace rather than add, `test_a_second_deposit_replaces`; every
//! other row n/a.

use deep_causality_context_store::utils_test::{MemorySubstrate, block_on};
use deep_causality_context_store::{DataRecord, Substrate};

#[test]
fn test_a_new_substrate_is_empty() {
    let substrate = MemorySubstrate::new();
    assert!(substrate.is_empty());
    assert_eq!(substrate.len(), 0);
    assert!(MemorySubstrate::default().is_empty());
    assert_eq!(MemorySubstrate::SOURCE, "memory");
}

#[test]
fn test_a_second_deposit_replaces() {
    let substrate = MemorySubstrate::new();
    let first = block_on(substrate.deposit(4, &DataRecord::Count(1))).unwrap();
    let second = block_on(substrate.deposit(4, &DataRecord::Count(2))).unwrap();
    assert_eq!(first, second);
    assert_eq!(substrate.len(), 1);
    assert!(!substrate.is_empty());
    assert_eq!(
        block_on(substrate.resolve(&second)),
        Ok(DataRecord::Count(2))
    );
    block_on(substrate.deposit(5, &DataRecord::Count(3))).unwrap();
    assert_eq!(substrate.len(), 2);
}
