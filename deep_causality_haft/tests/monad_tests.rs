/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use deep_causality_haft::{Monad, OptionWitness, ResultWitness};

#[test]
fn test_monad_option() {
    let opt_a = Some(5);
    let f = |x| Some(x * 2);
    let opt_b = OptionWitness::bind(opt_a, f);
    assert_eq!(opt_b, Some(10));

    let opt_none: Option<i32> = None;
    let opt_none_bound = OptionWitness::bind(opt_none, f);
    assert_eq!(opt_none_bound, None);

    let opt_a_to_none = Some(5);
    let f_to_none = |_| -> Option<i32> { None };
    let opt_b_none = OptionWitness::bind(opt_a_to_none, f_to_none);
    assert_eq!(opt_b_none, None);

    let pure_val = OptionWitness::pure(100);
    assert_eq!(pure_val, Some(100));
}

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
