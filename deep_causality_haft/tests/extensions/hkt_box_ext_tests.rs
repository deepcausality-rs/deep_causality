/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Applicative, BoxWitness, Foldable, Functor, HKT, Monad};

// --- HKT Tests ---

#[test]
fn test_hkt_box_witness() {
    let value: <BoxWitness as HKT>::Type<i32> = Box::new(10);
    assert_eq!(value, Box::new(10));
}

// --- Functor Tests ---

#[test]
fn test_functor_box() {
    let box_a = Box::new(5);
    let f = |x| x * 2;
    let box_b = BoxWitness::fmap(box_a, f);
    assert_eq!(box_b, Box::new(10));
}

// --- Applicative Tests ---

#[test]
fn test_applicative_box_pure() {
    let b = BoxWitness::pure(10);
    assert_eq!(b, Box::new(10));
}

#[test]
fn test_applicative_box_apply() {
    let f_add_one = Box::new(|x| x + 1);
    let val = Box::new(10);
    let result = BoxWitness::apply(f_add_one, val);
    assert_eq!(result, Box::new(11));
}

// --- Foldable Tests ---

#[test]
fn test_foldable_box() {
    let b = Box::new(5);
    let result = BoxWitness::fold(b, 0, |acc, x| acc + x);
    assert_eq!(result, 5);
}

// --- Monad Tests ---

#[test]
fn test_monad_box() {
    let box_a = Box::new(5);
    let f = |x| Box::new(x * 2);
    let box_b = BoxWitness::bind(box_a, f);
    assert_eq!(box_b, Box::new(10));

    let pure_val = BoxWitness::pure(100);
    assert_eq!(pure_val, Box::new(100));
}
