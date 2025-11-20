/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{HKT, HKT2, HKT3, HKT4, HKT5, Placeholder};

// --- HKT (Arity 1) Tests ---

// Witness for Option
struct TestOptionWitness;

impl HKT for TestOptionWitness {
    type Type<T> = Option<T>;
}

#[test]
fn test_hkt_option() {
    let value: <TestOptionWitness as HKT>::Type<i32> = Some(10);
    assert_eq!(value, Some(10));

    let none_value: <TestOptionWitness as HKT>::Type<i32> = None;
    assert_eq!(none_value, None);
}

// Witness for Vec
struct TestVecWitness;

impl HKT for TestVecWitness {
    type Type<T> = Vec<T>;
}

#[test]
fn test_hkt_vec() {
    let value: <TestVecWitness as HKT>::Type<i32> = vec![1, 2, 3];
    assert_eq!(value, vec![1, 2, 3]);

    let empty_value: <TestVecWitness as HKT>::Type<i32> = Vec::new();
    assert_eq!(empty_value, Vec::<i32>::new());
}

// --- HKT2 (Arity 2) Tests ---

// Witness for Result<T, E> where E is fixed
struct TestResultWitness<E>(Placeholder, E);

impl<E> HKT2<E> for TestResultWitness<E> {
    type Type<T> = Result<T, E>;
}

#[test]
fn test_hkt2_result() {
    type MyResult<T> = <TestResultWitness<String> as HKT2<String>>::Type<T>;

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

// --- HKT4 (Arity 4) Tests ---

// A dummy type with four generic parameters to act as a witness
struct MyCustomType4<T, F1, F2, F3> {
    value: T,
    _f1: F1,
    _f2: F2,
    _f3: F3,
}

// Witness for MyCustomType4<T, F1, F2, F3> where F1, F2, and F3 are fixed
struct MyCustomTypeWitness4<F1, F2, F3>(Placeholder, F1, F2, F3);

impl<F1, F2, F3> HKT4<F1, F2, F3> for MyCustomTypeWitness4<F1, F2, F3> {
    type Type<T> = MyCustomType4<T, F1, F2, F3>;
}

#[test]
fn test_hkt4_custom_type() {
    type MyHkt4Type<T> =
        <MyCustomTypeWitness4<String, u32, bool> as HKT4<String, u32, bool>>::Type<T>;

    let instance = MyHkt4Type {
        value: 40,
        _f1: "Fixed String".to_string(),
        _f2: 200u32,
        _f3: true,
    };
    assert_eq!(instance.value, 40);
    assert_eq!(instance._f1, "Fixed String".to_string());
    assert_eq!(instance._f2, 200u32);
    assert!(instance._f3);
}

// --- HKT5 (Arity 5) Tests ---

// A dummy type with five generic parameters to act as a witness
struct MyCustomType5<T, F1, F2, F3, F4> {
    value: T,
    _f1: F1,
    _f2: F2,
    _f3: F3,
    _f4: F4,
}

// Witness for MyCustomType5<T, F1, F2, F3, F4> where F1, F2, F3, and F4 are fixed
struct MyCustomTypeWitness5<F1, F2, F3, F4>(Placeholder, F1, F2, F3, F4);

impl<F1, F2, F3, F4> HKT5<F1, F2, F3, F4> for MyCustomTypeWitness5<F1, F2, F3, F4> {
    type Type<T> = MyCustomType5<T, F1, F2, F3, F4>;
}

#[test]
fn test_hkt5_custom_type() {
    type MyHkt5Type<T> =
        <MyCustomTypeWitness5<String, u32, bool, f64> as HKT5<String, u32, bool, f64>>::Type<T>;

    let instance = MyHkt5Type {
        value: 50,
        _f1: "Fixed String".to_string(),
        _f2: 300u32,
        _f3: false,
        _f4: 1.23,
    };
    assert_eq!(instance.value, 50);
    assert_eq!(instance._f1, "Fixed String".to_string());
    assert_eq!(instance._f2, 300u32);
    assert!(!instance._f3);
    assert_eq!(instance._f4, 1.23);
}
