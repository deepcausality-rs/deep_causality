/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Foldable, OptionWitness, ResultWitness, VecWitness};

// --- OptionWitness Foldable Tests ---

#[test]
fn test_foldable_option_some() {
    let opt = Some(5);
    let result = OptionWitness::fold(opt, 0, |acc, x| acc + x);
    assert_eq!(result, 5);
}

#[test]
fn test_foldable_option_none() {
    let opt: Option<i32> = None;
    let result = OptionWitness::fold(opt, 0, |acc, x| acc + x);
    assert_eq!(result, 0);
}

#[test]
fn test_foldable_option_string_concat() {
    let opt = Some("hello".to_string());
    let result = OptionWitness::fold(opt, String::new(), |mut acc, x| {
        acc.push_str(&x);
        acc
    });
    assert_eq!(result, "hello");
}

// --- ResultWitness Foldable Tests ---

#[test]
fn test_foldable_result_ok() {
    let res: Result<i32, String> = Ok(5);
    let result = ResultWitness::fold(res, 0, |acc, x| acc + x);
    assert_eq!(result, 5);
}

#[test]
fn test_foldable_result_err() {
    let res: Result<i32, String> = Err("error".to_string());
    let result = ResultWitness::fold(res, 0, |acc, x| acc + x);
    assert_eq!(result, 0);
}

#[test]
fn test_foldable_result_string_concat() {
    let res: Result<String, String> = Ok("world".to_string());
    let result = ResultWitness::fold(res, String::new(), |mut acc, x| {
        acc.push_str(&x);
        acc
    });
    assert_eq!(result, "world");
}

// --- VecWitness Foldable Tests ---

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
