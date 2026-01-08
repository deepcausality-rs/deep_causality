/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::collections::HashMap;

#[test]
fn test_len() {
    use deep_causality_macros::make_len;

    #[allow(dead_code)]
    trait TestLenTrait<T> {
        fn len(&self) -> usize;
    }

    impl TestLenTrait<i32> for Vec<i32> {
        make_len!();
    }

    let test = [1, 2, 3];
    assert_eq!(test.len(), 3);
}

#[test]
fn test_is_empty() {
    use deep_causality_macros::make_is_empty;

    #[allow(dead_code)]
    trait TestIsEmptyTrait<T> {
        fn is_empty(&self) -> bool;
    }

    impl<T> TestIsEmptyTrait<T> for Vec<T>
    where
        T: Clone,
    {
        make_is_empty!();
    }

    let empty: Vec<i32> = vec![];
    assert!(empty.is_empty());
}

#[test]
fn test_get_all_items() {
    use deep_causality_macros::make_get_all_items;

    #[allow(dead_code)]
    trait TestGetAllItemsTrait<T> {
        fn get_all_items(&self) -> Vec<&T>;
    }

    impl<T> TestGetAllItemsTrait<T> for Vec<T>
    where
        T: Clone,
    {
        make_get_all_items!();
    }

    let test = vec![1, 2, 3];
    let items: Vec<&i32> = test.get_all_items();
    assert_eq!(items, vec![&1, &2, &3]);
}

#[test]
fn test_to_vec() {
    use deep_causality_macros::make_vec_to_vec;

    #[allow(dead_code)]
    trait TestToVecTrait<T> {
        fn to_vec(&self) -> Vec<T>;
    }

    impl<T> TestToVecTrait<T> for Vec<T>
    where
        T: Clone,
    {
        make_vec_to_vec!();
    }

    let test = vec![1, 2, 3];
    let vec: Vec<i32> = test.to_vec();
    assert_eq!(vec, vec![1, 2, 3]);
}

#[test]
fn test_array_to_vec() {
    use deep_causality_macros::make_array_to_vec;

    #[allow(dead_code)]
    trait TestArrayToVec<T> {
        fn to_vec(&self) -> Vec<T>;
    }

    impl<T: Clone> TestArrayToVec<T> for [T] {
        make_array_to_vec!();
    }

    let test: [i32; 3] = [1, 2, 3];
    let vec = test.to_vec();
    assert_eq!(vec, vec![1, 2, 3]);
}

// Define a mock Identifiable trait and struct for testing find macros
trait Identifiable {
    fn id(&self) -> u64;
}

type IdentificationValue = u64;

#[derive(Clone, Copy, Debug, PartialEq)]
struct MockItem {
    id: u64,
    value: i32,
}

impl Identifiable for MockItem {
    fn id(&self) -> u64 {
        self.id
    }
}

#[test]
fn test_find_from_iter_values() {
    use deep_causality_macros::make_find_from_iter_values;

    trait Findable<T> {
        fn get_item_by_id(&self, id: IdentificationValue) -> Option<&T>;
    }

    impl<T: Identifiable> Findable<T> for Vec<T> {
        make_find_from_iter_values!();
    }

    let items = vec![MockItem { id: 1, value: 10 }, MockItem { id: 2, value: 20 }];

    let found = items.get_item_by_id(2).unwrap();
    assert_eq!(*found, MockItem { id: 2, value: 20 });

    let not_found = items.get_item_by_id(3);
    assert!(not_found.is_none());
}

#[test]
fn test_find_from_map_values() {
    use deep_causality_macros::make_find_from_map_values;

    trait FindableInMap<V> {
        fn get_item_by_id(&self, id: IdentificationValue) -> Option<&V>;
    }

    impl<K, V: Identifiable> FindableInMap<V> for HashMap<K, V> {
        make_find_from_map_values!();
    }

    let mut map = HashMap::new();
    map.insert("a", MockItem { id: 1, value: 10 });
    map.insert("b", MockItem { id: 2, value: 20 });

    let found = map.get_item_by_id(2).unwrap();
    assert_eq!(*found, MockItem { id: 2, value: 20 });

    let not_found = map.get_item_by_id(3);
    assert!(not_found.is_none());
}
