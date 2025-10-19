/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Functor, HKT, Monad, VecWitness};

// --- HKT Tests ---

#[test]
fn test_hkt_vec_witness() {
    let value: <VecWitness as HKT>::Type<i32> = vec![1, 2, 3];
    assert_eq!(value, vec![1, 2, 3]);

    let empty_value: <VecWitness as HKT>::Type<i32> = Vec::new();
    assert_eq!(empty_value, Vec::<i32>::new());
}

// --- Functor Tests ---

#[test]
fn test_functor_vec() {
    let vec_a = vec![1, 2, 3];
    let f = |x| x * 2;
    let vec_b = VecWitness::fmap(vec_a, f);
    assert_eq!(vec_b, vec![2, 4, 6]);

    let vec_empty: Vec<i32> = Vec::new();
    let f_empty = |x: i32| x * 2;
    let vec_empty_mapped = VecWitness::fmap(vec_empty, f_empty);
    assert_eq!(vec_empty_mapped, Vec::<i32>::new());
}

// --- Monad Tests ---

#[test]
fn test_monad_vec() {
    let vec_a = vec![1, 2, 3];
    let f = |x| vec![x, x * 10];
    let vec_b = VecWitness::bind(vec_a, f);
    assert_eq!(vec_b, vec![1, 10, 2, 20, 3, 30]);

    let pure_val = VecWitness::pure(100);
    assert_eq!(pure_val, vec![100]);

    let vec_empty: Vec<i32> = Vec::new();
    let f_empty = |x: i32| vec![x, x * 10];
    let vec_empty_bound = VecWitness::bind(vec_empty, f_empty);
    assert_eq!(vec_empty_bound, Vec::<i32>::new());
}
