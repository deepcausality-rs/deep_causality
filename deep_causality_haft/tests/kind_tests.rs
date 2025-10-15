/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{HKT, HKT2, HKT3, Placeholder};

// --- HKT (Arity 1) Tests ---

// Witness for Option
struct OptionWitness;

impl HKT for OptionWitness {
    type Type<T> = Option<T>;
}

#[test]
fn test_hkt_option() {
    let value: <OptionWitness as HKT>::Type<i32> = Some(10);
    assert_eq!(value, Some(10));

    let none_value: <OptionWitness as HKT>::Type<i32> = None;
    assert_eq!(none_value, None);
}

// Witness for Vec
struct VecWitness;

impl HKT for VecWitness {
    type Type<T> = Vec<T>;
}

#[test]
fn test_hkt_vec() {
    let value: <VecWitness as HKT>::Type<i32> = vec![1, 2, 3];
    assert_eq!(value, vec![1, 2, 3]);

    let empty_value: <VecWitness as HKT>::Type<i32> = Vec::new();
    assert_eq!(empty_value, Vec::<i32>::new());
}

// --- HKT2 (Arity 2) Tests ---

// Witness for Result<T, E> where E is fixed
struct ResultWitness<E>(Placeholder, E);

impl<E> HKT2<E> for ResultWitness<E> {
    type Type<T> = Result<T, E>;
}

#[test]
fn test_hkt2_result() {
    type MyResult<T> = <ResultWitness<String> as HKT2<String>>::Type<T>;

    let ok_value: MyResult<i32> = Ok(20);
    assert_eq!(ok_value, Ok(20));

    let err_value: MyResult<i32> = Err("Error occurred".to_string());
    assert_eq!(err_value, Err("Error occurred".to_string()));
}

// --- HKT3 (Arity 3) Tests ---

// A dummy type with three generic parameters to act as a witness
struct MyCustomType<T, F1, F2> {
    value: T,
    _f1: F1,
    _f2: F2,
}

// Witness for MyCustomType<T, F1, F2> where F1 and F2 are fixed
struct MyCustomTypeWitness<F1, F2>(Placeholder, F1, F2);

impl<F1, F2> HKT3<F1, F2> for MyCustomTypeWitness<F1, F2> {
    type Type<T> = MyCustomType<T, F1, F2>;
}

#[test]
fn test_hkt3_custom_type() {
    type MyHkt3Type<T> = <MyCustomTypeWitness<String, u32> as HKT3<String, u32>>::Type<T>;

    let instance = MyHkt3Type {
        value: 30,
        _f1: "Fixed String".to_string(),
        _f2: 100u32,
    };
    assert_eq!(instance.value, 30);
    assert_eq!(instance._f1, "Fixed String".to_string());
    assert_eq!(instance._f2, 100u32);
}
