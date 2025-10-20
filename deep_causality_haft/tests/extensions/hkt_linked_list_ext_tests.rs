/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Foldable, Functor, HKT, LinkedListWitness, Monad};
use std::collections::LinkedList;

// Helper to create a LinkedList from a Vec
fn from_vec<T>(vec: Vec<T>) -> LinkedList<T> {
    vec.into_iter().collect()
}

// --- HKT Tests ---

#[test]
fn test_hkt_linked_list_witness() {
    let list: <LinkedListWitness as HKT>::Type<i32> = from_vec(vec![1, 2, 3]);
    assert_eq!(list.front(), Some(&1));
}

// --- Functor Tests ---

#[test]
fn test_functor_linked_list() {
    let list_a = from_vec(vec![1, 2, 3]);
    let f = |x| x * 2;
    let list_b = LinkedListWitness::fmap(list_a, f);
    assert_eq!(list_b, from_vec(vec![2, 4, 6]));
}

// --- Foldable Tests ---

#[test]
fn test_foldable_linked_list() {
    let list = from_vec(vec![1, 2, 3]);
    let result = LinkedListWitness::fold(list, 0, |acc, x| acc + x);
    assert_eq!(result, 6);
}

// --- Applicative Tests ---

#[test]
fn test_applicative_linked_list_pure() {
    let list = LinkedListWitness::pure(10);
    assert_eq!(list, from_vec(vec![10]));
}

#[test]
fn test_applicative_linked_list_apply() {
    let f_funcs = from_vec(vec![|x| x + 1, |x| x * 2]);
    let vals = from_vec(vec![10, 20]);
    let result = LinkedListWitness::apply(f_funcs, vals);
    // Expected: [(10+1), (20+1), (10*2), (20*2)]
    assert_eq!(result, from_vec(vec![11, 21, 20, 40]));
}

// --- Monad Tests ---

#[test]
fn test_monad_linked_list_bind() {
    let list_a = from_vec(vec![1, 2, 3]);
    let f = |x| from_vec(vec![x, x * 10]);
    let list_b = LinkedListWitness::bind(list_a, f);
    assert_eq!(list_b, from_vec(vec![1, 10, 2, 20, 3, 30]));
}
