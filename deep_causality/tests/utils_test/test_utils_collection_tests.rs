/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_collection::{
    get_large_collection_and_data, get_medium_collection_and_data, get_small_collection_and_data,
};

#[test]
fn test_get_small_collection_and_data() {
    let (collection, data) = get_small_collection_and_data();
    assert_eq!(collection.len(), 10);
    assert_eq!(data.len(), 10);
    assert!(data.iter().all(|&x| x == 0.99));
}

#[test]
fn test_get_medium_collection_and_data() {
    let (collection, data) = get_medium_collection_and_data();
    assert_eq!(collection.len(), 1_000);
    assert_eq!(data.len(), 1_000);
    assert!(data.iter().all(|&x| x == 0.99));
}

#[test]
fn test_get_large_collection_and_data() {
    let (collection, data) = get_large_collection_and_data();
    assert_eq!(collection.len(), 10_000);
    assert_eq!(data.len(), 10_000);
    assert!(data.iter().all(|&x| x == 0.99));
}
