/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, Foldable, Functor, HKT, Monad, OptionWitness};

// --- Applicative Tests ---

#[test]
fn test_applicative_option_pure() {
    let opt = OptionWitness::pure(10);
    assert_eq!(opt, Some(10));
}

#[test]
fn test_applicative_option_apply_some() {
    let f_add_one = Some(|x| x + 1);
    let val = Some(10);
    let result = OptionWitness::apply(f_add_one, val);
    assert_eq!(result, Some(11));
}

#[test]
fn test_applicative_option_apply_none_func() {
    let f_add_one: Option<fn(i32) -> i32> = None;
    let val = Some(10);
    let result = OptionWitness::apply(f_add_one, val);
    assert_eq!(result, None);
}

#[test]
fn test_applicative_option_apply_none_val() {
    let f_add_one = Some(|x| x + 1);
    let val: Option<i32> = None;
    let result = OptionWitness::apply(f_add_one, val);
    assert_eq!(result, None);
}

// --- Foldable Tests ---

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

// --- HKT Tests ---

#[test]
fn test_hkt_option_witness() {
    let value: <OptionWitness as HKT>::Type<i32> = Some(10);
    assert_eq!(value, Some(10));

    let none_value: <OptionWitness as HKT>::Type<i32> = None;
    assert_eq!(none_value, None);
}

// --- Functor Tests ---

#[test]
fn test_functor_option() {
    let opt_a = Some(5);
    let f = |x| x * 2;
    let opt_b = OptionWitness::fmap(opt_a, f);
    assert_eq!(opt_b, Some(10));

    let opt_none: Option<i32> = None;
    let opt_none_mapped = OptionWitness::fmap(opt_none, f);
    assert_eq!(opt_none_mapped, None);
}

// --- Monad Tests ---

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
