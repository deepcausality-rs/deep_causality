/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

// NOTE: VecWitness::sequence tests are temporarily disabled.
// The Traversable implementation for VecWitness was removed due to
// constraint system complexity with closures.
// See hkt_vec_ext.rs for details.

use deep_causality_haft::{OptionWitness, ResultWitness, Traversable};

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
