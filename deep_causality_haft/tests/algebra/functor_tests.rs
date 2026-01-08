/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Functor, OptionWitness, ResultWitness, VecWitness};

#[test]
fn test_functor_option() {
    let some_v = Some(2);
    let mapped = OptionWitness::fmap(some_v, |x| x * 2);
    assert_eq!(mapped, Some(4));

    let none_v: Option<i32> = None;
    let mapped_none = OptionWitness::fmap(none_v, |x| x * 2);
    assert_eq!(mapped_none, None);
}

#[test]
fn test_functor_result() {
    let ok_v: Result<i32, &str> = Ok(2);
    let mapped_ok = ResultWitness::fmap(ok_v, |x| x * 2);
    assert_eq!(mapped_ok, Ok(4));

    let err_v: Result<i32, &str> = Err("error");
    let mapped_err = ResultWitness::fmap(err_v, |x| x * 2);
    assert_eq!(mapped_err, Err("error"));
}

#[test]
fn test_functor_vec() {
    let v = vec![1, 2, 3];
    let mapped_v = VecWitness::fmap(v, |x| x * 2);
    assert_eq!(mapped_v, vec![2, 4, 6]);
}
