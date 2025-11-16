/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality::utils_test::test_utils_map::{
    get_large_map_and_data, get_medium_map_and_data, get_small_map_and_data,
};

#[test]
fn test_get_small_map_and_data() {
    let (causal_map, data_map) = get_small_map_and_data();
    assert_eq!(causal_map.len(), 10);
    assert_eq!(data_map.len(), 10);
    for i in 0..10 {
        assert!(causal_map.contains_key(&i));
        assert!(data_map.contains_key(&i));
        assert_eq!(*data_map.get(&i).unwrap(), 0.99);
    }
}

#[test]
fn test_get_medium_map_and_data() {
    let (causal_map, data_map) = get_medium_map_and_data();
    assert_eq!(causal_map.len(), 1_000);
    assert_eq!(data_map.len(), 1_000);
    for i in 0..1_000 {
        assert!(causal_map.contains_key(&i));
        assert!(data_map.contains_key(&i));
        assert_eq!(*data_map.get(&i).unwrap(), 0.99);
    }
}

#[test]
fn test_get_large_map_and_data() {
    let (causal_map, data_map) = get_large_map_and_data();
    assert_eq!(causal_map.len(), 10_000);
    assert_eq!(data_map.len(), 10_000);
    for i in 0..10_000 {
        assert!(causal_map.contains_key(&i));
        assert!(data_map.contains_key(&i));
        assert_eq!(*data_map.get(&i).unwrap(), 0.99);
    }
}
