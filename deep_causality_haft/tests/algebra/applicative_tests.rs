/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, OptionWitness, Pure, ResultWitness};

#[test]
fn test_applicative_option() {
    let pure_val = OptionWitness::pure(5);
    assert_eq!(pure_val, Some(5));

    let func = Some(|x: i32| x * 2);
    let val = Some(10);
    let applied = OptionWitness::apply(func, val);
    assert_eq!(applied, Some(20));

    let none_func: Option<fn(i32) -> i32> = None;
    let applied_none_func = OptionWitness::apply(none_func, val);
    assert_eq!(applied_none_func, None);

    let none_val: Option<i32> = None;
    let applied_none_val = OptionWitness::apply(func, none_val);
    assert_eq!(applied_none_val, None);
}

#[test]
fn test_applicative_result() {
    let pure_val: Result<i32, &str> = ResultWitness::pure(5);
    assert_eq!(pure_val, Ok(5));

    let func: Result<fn(i32) -> i32, &str> = Ok(|x| x * 2);
    let val: Result<i32, &str> = Ok(10);
    let applied = ResultWitness::apply(func, val);
    assert_eq!(applied, Ok(20));

    let err_func: Result<fn(i32) -> i32, &str> = Err("func error");
    let applied_err_func = ResultWitness::apply(err_func, val);
    assert_eq!(applied_err_func, Err("func error"));

    let err_val: Result<i32, &str> = Err("val error");
    let applied_err_val = ResultWitness::apply(func, err_val);
    assert_eq!(applied_err_val, Err("val error"));
}
