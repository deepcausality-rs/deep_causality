/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::Bifunctor;
use deep_causality_haft::ResultUnboundWitness;

#[test]
fn test_result_bimap() {
    let ok: Result<i32, &str> = Ok(10);
    let err: Result<i32, &str> = Err("error");

    let res_ok = ResultUnboundWitness::bimap(ok, |x| x * 2, |e| e.len());
    assert_eq!(res_ok, Ok(20));

    let res_err = ResultUnboundWitness::bimap(err, |x| x * 2, |e| e.len());
    assert_eq!(res_err, Err(5));
}

#[test]
fn test_result_first() {
    let ok: Result<i32, &str> = Ok(10);
    let err: Result<i32, &str> = Err("error");

    let res_ok = ResultUnboundWitness::first(ok, |x| x * 2);
    assert_eq!(res_ok, Ok(20));

    let res_err = ResultUnboundWitness::first(err, |x| x * 2);
    assert_eq!(res_err, Err("error"));
}

#[test]
fn test_result_second() {
    let ok: Result<i32, &str> = Ok(10);
    let err: Result<i32, &str> = Err("error");

    let res_ok = ResultUnboundWitness::second(ok, |e| e.len());
    assert_eq!(res_ok, Ok(10));

    let res_err = ResultUnboundWitness::second(err, |e| e.len());
    assert_eq!(res_err, Err(5));
}
