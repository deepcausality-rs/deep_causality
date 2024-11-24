// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

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

    let test = [1, 2, 3];
    assert!(!test.is_empty());

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
