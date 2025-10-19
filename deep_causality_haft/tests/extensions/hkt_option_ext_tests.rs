/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Functor, HKT, Monad, OptionWitness};

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
