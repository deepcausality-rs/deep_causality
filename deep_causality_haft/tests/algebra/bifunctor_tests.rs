/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Bifunctor, ResultUnboundWitness};

#[test]
fn test_bifunctor_result() {
    let ok_v: Result<i32, String> = Ok(10);
    let bimapped_ok = ResultUnboundWitness::bimap(ok_v, |v| v * 2, |e| e);
    assert_eq!(bimapped_ok, Ok(20));

    let err_v: Result<i32, String> = Err("error".to_string());
    let bimapped_err = ResultUnboundWitness::bimap(err_v, |v| v * 2, |e| format!("new {}", e));
    assert_eq!(bimapped_err, Err("new error".to_string()));

    let err_v2: Result<i32, String> = Err("error".to_string());
    let first_mapped = ResultUnboundWitness::first(err_v2, |v| v + 5);
    assert_eq!(first_mapped, Err("error".to_string()));

    let ok_v2: Result<i32, String> = Ok(10);
    let second_mapped = ResultUnboundWitness::second(ok_v2, |e| format!("mapped {}", e));
    assert_eq!(second_mapped, Ok(10));
}
