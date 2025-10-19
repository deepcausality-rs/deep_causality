/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Functor, HKT, Monad, VecWitness};

// --- Applicative Tests ---

#[test]
fn test_applicative_vec_pure() {
    let vec = VecWitness::pure(10);
    assert_eq!(vec, vec![10]);
}

#[test]
fn test_applicative_vec_apply_non_empty() {
    let f_funcs = vec![|x| x + 1, |x| x * 2];
    let vals = vec![10, 20];
    let result = VecWitness::apply(f_funcs, vals);
    // Expected: [(10+1), (20+1), (10*2), (20*2)] = [11, 21, 20, 40]
    assert_eq!(result, vec![11, 21, 20, 40]);
}

#[test]
fn test_applicative_vec_apply_empty_func() {
    let f_funcs: Vec<fn(i32) -> i32> = Vec::new();
    let vals = vec![10, 20];
    let result = VecWitness::apply(f_funcs, vals);
    assert_eq!(result, Vec::<i32>::new());
}

#[test]
fn test_applicative_vec_apply_empty_val() {
    let f_funcs = vec![|x| x + 1, |x| x * 2];
    let vals: Vec<i32> = Vec::new();
    let result = VecWitness::apply(f_funcs, vals);
    assert_eq!(result, Vec::<i32>::new());
}

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
