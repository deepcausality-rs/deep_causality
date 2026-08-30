/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Foldable, Functor, HKT, HKT2, HashMapWitness};
use std::collections::HashMap;

// --- HKT Tests ---

#[test]
fn test_hkt_hash_map_witness() {
    type MyMap<V> = <HashMapWitness<String> as HKT>::Type<V>;
    let mut map: MyMap<i32> = HashMap::new();
    map.insert("one".to_string(), 1);
    assert_eq!(map.get("one"), Some(&1));

    type MyMap2<V> = <HashMapWitness<String> as HKT2<String>>::Type<V>;
    let mut map2: MyMap2<i32> = HashMap::new();
    map2.insert("two".to_string(), 2);
    assert_eq!(map2.get("two"), Some(&2));
}

// --- Functor Tests ---

#[test]
fn test_functor_hash_map() {
    let mut map_a: HashMap<String, i32> = HashMap::new();
    map_a.insert("a".to_string(), 1);
    map_a.insert("b".to_string(), 2);

    let f = |x| x * 2;
    let map_b = HashMapWitness::fmap(map_a, f);

    let mut expected_map: HashMap<String, i32> = HashMap::new();
    expected_map.insert("a".to_string(), 2);
    expected_map.insert("b".to_string(), 4);

    assert_eq!(map_b, expected_map);
}

#[test]
fn test_functor_hash_map_empty() {
    let map_a: HashMap<String, i32> = HashMap::new();
    let f = |x| x * 2;
    let map_b = HashMapWitness::fmap(map_a, f);
    assert!(map_b.is_empty());
}

#[test]
fn test_functor_hash_map_type_change() {
    let mut map_a: HashMap<String, i32> = HashMap::new();
    map_a.insert("key".to_string(), 10);

    let f = |x: i32| x.to_string();
    let map_b = HashMapWitness::fmap(map_a, f);

    let mut expected_map: HashMap<String, String> = HashMap::new();
    expected_map.insert("key".to_string(), "10".to_string());

    assert_eq!(map_b, expected_map);
}

// --- Foldable Tests ---

#[test]
fn test_foldable_hash_map_sum_values() {
    let mut map: HashMap<String, i32> = HashMap::new();
    map.insert("one".to_string(), 1);
    map.insert("two".to_string(), 2);
    map.insert("three".to_string(), 3);

    let sum = HashMapWitness::fold(map, 0, |acc, v| acc + v);
    assert_eq!(sum, 6);
}

#[test]
fn test_foldable_hash_map_empty() {
    let map: HashMap<String, i32> = HashMap::new();
    let sum = HashMapWitness::fold(map, 0, |acc, v| acc + v);
    assert_eq!(sum, 0);
}
