/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Foldable, Functor, HKT, VecDequeWitness};
use std::collections::VecDeque;

// --- HKT Tests ---

#[test]
fn test_hkt_vec_deque_witness() {
    let value: <VecDequeWitness as HKT>::Type<i32> = VecDeque::from(vec![1, 2, 3]);
    assert_eq!(value, VecDeque::from(vec![1, 2, 3]));

    let empty_value: <VecDequeWitness as HKT>::Type<i32> = VecDeque::new();
    assert_eq!(empty_value, VecDeque::<i32>::new());
}

// --- Functor Tests ---

#[test]
fn test_functor_vec_deque() {
    let deque_a = VecDeque::from(vec![1, 2, 3]);
    let f = |x| x * 2;
    let deque_b = VecDequeWitness::fmap(deque_a, f);
    assert_eq!(deque_b, VecDeque::from(vec![2, 4, 6]));
}

#[test]
fn test_functor_vec_deque_empty() {
    let deque_a: VecDeque<i32> = VecDeque::new();
    let f = |x| x * 2;
    let deque_b = VecDequeWitness::fmap(deque_a, f);
    assert!(deque_b.is_empty());
}

#[test]
fn test_functor_vec_deque_type_change() {
    let deque_a = VecDeque::from(vec![10, 20]);
    let f = |x: i32| x.to_string();
    let deque_b = VecDequeWitness::fmap(deque_a, f);
    assert_eq!(
        deque_b,
        VecDeque::from(vec!["10".to_string(), "20".to_string()])
    );
}

// --- Foldable Tests ---

#[test]
fn test_foldable_vec_deque_sum() {
    let deque = VecDeque::from(vec![1, 2, 3, 4, 5]);
    let sum = VecDequeWitness::fold(deque, 0, |acc, x| acc + x);
    assert_eq!(sum, 15);
}

#[test]
fn test_foldable_vec_deque_concat() {
    let deque = VecDeque::from(vec!["hello".to_string(), "world".to_string()]);
    let concatenated = VecDequeWitness::fold(deque, String::new(), |mut acc, x| {
        if !acc.is_empty() {
            acc.push(' ');
        }
        acc.push_str(&x);
        acc
    });
    assert_eq!(concatenated, "hello world");
}

#[test]
fn test_foldable_vec_deque_empty() {
    let deque: VecDeque<i32> = VecDeque::new();
    let sum = VecDequeWitness::fold(deque, 0, |acc, x| acc + x);
    assert_eq!(sum, 0);
}
