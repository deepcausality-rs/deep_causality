/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::{Functor, HKT, HKT2, Monad, ResultWitness};

// --- HKT Tests ---

#[test]
fn test_hkt_result_witness() {
    type MyResult<T> = <ResultWitness<String> as HKT>::Type<T>;

    let ok_value: MyResult<i32> = Ok(10);
    assert_eq!(ok_value, Ok(10));

    let err_value: MyResult<i32> = Err("Error occurred".to_string());
    assert_eq!(err_value, Err("Error occurred".to_string()));
}

#[test]
fn test_hkt2_result_witness() {
    type MyResult<T> = <ResultWitness<String> as HKT2<String>>::Type<T>;

    let ok_value: MyResult<i32> = Ok(20);
    assert_eq!(ok_value, Ok(20));

    let err_value: MyResult<i32> = Err("Error occurred".to_string());
    assert_eq!(err_value, Err("Error occurred".to_string()));
}

// --- Functor Tests ---

#[test]
fn test_functor_result() {
    let res_a: Result<i32, String> = Ok(5);
    let f = |x| x * 2;
    let res_b = ResultWitness::fmap(res_a, f);
    assert_eq!(res_b, Ok(10));

    let res_err: Result<i32, String> = Err("Error".to_string());
    let res_err_mapped = ResultWitness::fmap(res_err, f);
    assert_eq!(res_err_mapped, Err("Error".to_string()));
}

// --- Monad Tests ---

#[test]
fn test_monad_result() {
    let res_a: Result<i32, String> = Ok(5);
    let f = |x| Ok(x * 2);
    let res_b = ResultWitness::bind(res_a, f);
    assert_eq!(res_b, Ok(10));

    let res_err: Result<i32, String> = Err("Error".to_string());
    let res_err_bound = ResultWitness::bind(res_err, f);
    assert_eq!(res_err_bound, Err("Error".to_string()));

    let res_a_to_err: Result<i32, String> = Ok(5);
    let f_to_err = |_| -> Result<i32, String> { Err("Inner Error".to_string()) };
    let res_b_err = ResultWitness::bind(res_a_to_err, f_to_err);
    assert_eq!(res_b_err, Err("Inner Error".to_string()));

    let pure_val: Result<i32, String> = ResultWitness::pure(100);
    assert_eq!(pure_val, Ok(100));
}
