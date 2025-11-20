/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{
    Applicative, Foldable, Functor, HKT, Monad, OptionWitness, ResultWitness, Traversable,
    VecWitness,
};

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

// --- Foldable Tests ---

#[test]
fn test_foldable_vec_non_empty() {
    let vec = vec![1, 2, 3];
    let result = VecWitness::fold(vec, 0, |acc, x| acc + x);
    assert_eq!(result, 6);
}

#[test]
fn test_foldable_vec_empty() {
    let vec: Vec<i32> = Vec::new();
    let result = VecWitness::fold(vec, 0, |acc, x| acc + x);
    assert_eq!(result, 0);
}

#[test]
fn test_foldable_vec_string_concat() {
    let vec = vec!["hello".to_string(), " ".to_string(), "world".to_string()];
    let result = VecWitness::fold(vec, String::new(), |mut acc, x| {
        acc.push_str(&x);
        acc
    });
    assert_eq!(result, "hello world");
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

// --- Traversable Tests ---

#[test]
fn test_traversable_vec_sequence_option() {
    type InnerMonad = OptionWitness;
    let vec_opt: Vec<Option<i32>> = vec![Some(1), Some(2), Some(3)];
    let sequenced: Option<Vec<i32>> = VecWitness::sequence::<i32, InnerMonad>(vec_opt);
    assert_eq!(sequenced, Some(vec![1, 2, 3]));

    let vec_opt_with_none: Vec<Option<i32>> = vec![Some(1), None, Some(3)];
    let sequenced_none: Option<Vec<i32>> =
        VecWitness::sequence::<i32, InnerMonad>(vec_opt_with_none);
    assert_eq!(sequenced_none, None);

    let empty_vec_opt: Vec<Option<i32>> = vec![];
    let sequenced_empty: Option<Vec<i32>> = VecWitness::sequence::<i32, InnerMonad>(empty_vec_opt);
    assert_eq!(sequenced_empty, Some(vec![]));
}

#[test]
fn test_traversable_vec_sequence_result() {
    type InnerMonad<E> = ResultWitness<E>;
    let vec_res: Vec<Result<i32, String>> = vec![Ok(1), Ok(2), Ok(3)];
    let sequenced: Result<Vec<i32>, String> =
        VecWitness::sequence::<i32, InnerMonad<String>>(vec_res);
    assert_eq!(sequenced, Ok(vec![1, 2, 3]));

    let vec_res_with_err: Vec<Result<i32, String>> = vec![Ok(1), Err("Error!".to_string()), Ok(3)];
    let sequenced_err: Result<Vec<i32>, String> =
        VecWitness::sequence::<i32, InnerMonad<String>>(vec_res_with_err);
    assert_eq!(sequenced_err, Err("Error!".to_string()));

    let empty_vec_res: Vec<Result<i32, String>> = vec![];
    let sequenced_empty_res: Result<Vec<i32>, String> =
        VecWitness::sequence::<i32, InnerMonad<String>>(empty_vec_res);
    assert_eq!(sequenced_empty_res, Ok(vec![]));
}
