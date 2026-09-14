/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{OptionWitness, ResultWitness, Traversable, VecWitness};

#[test]
fn test_traversable_option_result() {
    // Sequence: Option<Result<T, E>> -> Result<Option<T>, E>
    let opt_ok: Option<Result<i32, String>> = Some(Ok(42));
    let result_opt = OptionWitness::sequence::<i32, ResultWitness<String>>(opt_ok);
    assert_eq!(result_opt, Ok(Some(42)));

    let opt_err: Option<Result<i32, String>> = Some(Err("error".to_string()));
    let result_err = OptionWitness::sequence::<i32, ResultWitness<String>>(opt_err);
    assert_eq!(result_err, Err("error".to_string()));

    let opt_none: Option<Result<i32, String>> = None;
    let result_none = OptionWitness::sequence::<i32, ResultWitness<String>>(opt_none);
    assert_eq!(result_none, Ok(None));
}

#[test]
fn test_traversable_result_option() {
    // Sequence: Result<Option<T>, E> -> Option<Result<T, E>>
    let res_some: Result<Option<i32>, String> = Ok(Some(42));
    let opt_res = ResultWitness::sequence::<i32, OptionWitness>(res_some);
    assert_eq!(opt_res, Some(Ok(42)));

    let res_none: Result<Option<i32>, String> = Ok(None);
    let opt_res_none = ResultWitness::sequence::<i32, OptionWitness>(res_none);
    assert_eq!(opt_res_none, None);

    let res_err: Result<Option<i32>, String> = Err("error".to_string());
    let opt_err = ResultWitness::sequence::<i32, OptionWitness>(res_err);
    assert_eq!(opt_err, Some(Err("error".to_string())));
}

#[test]
fn test_traversable_vec_result() {
    // Sequence: Vec<Result<T, E>> -> Result<Vec<T>, E>
    let all_ok: Vec<Result<i32, String>> = vec![Ok(1), Ok(2), Ok(3)];
    assert_eq!(
        VecWitness::sequence::<i32, ResultWitness<String>>(all_ok),
        Ok(vec![1, 2, 3])
    );

    let with_err: Vec<Result<i32, String>> = vec![Ok(1), Err("boom".to_string()), Ok(3)];
    assert_eq!(
        VecWitness::sequence::<i32, ResultWitness<String>>(with_err),
        Err("boom".to_string())
    );

    let empty: Vec<Result<i32, String>> = Vec::new();
    assert_eq!(
        VecWitness::sequence::<i32, ResultWitness<String>>(empty),
        Ok(Vec::new())
    );
}

#[test]
fn test_traversable_vec_option() {
    // Sequence: Vec<Option<T>> -> Option<Vec<T>>
    let all_some: Vec<Option<i32>> = vec![Some(1), Some(2)];
    assert_eq!(
        VecWitness::sequence::<i32, OptionWitness>(all_some),
        Some(vec![1, 2])
    );

    let with_none: Vec<Option<i32>> = vec![Some(1), None];
    assert_eq!(VecWitness::sequence::<i32, OptionWitness>(with_none), None);
}
