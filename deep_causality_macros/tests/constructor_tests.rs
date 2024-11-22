// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::Constructor;

#[test]
fn test_derive_struct() {
    #[derive(Constructor)]
    struct Test {
        field1: i32,
        field2: String,
    }

    let test = Test::new(42, "test".to_string());
    assert_eq!(test.field1, 42);
    assert_eq!(test.field2, "test");
}

#[test]
fn test_derive_enum() {
    #[derive(Constructor)]
    enum TestEnum {
        Variant1(i32),
        Variant2 { field: String },
    }

    let v1 = TestEnum::new_variant1(42);
    match v1 {
        TestEnum::Variant1(val) => assert_eq!(val, 42),
        _ => panic!("Wrong variant"),
    }

    let v2 = TestEnum::new_variant2("test".to_string());
    match v2 {
        TestEnum::Variant2 { field } => assert_eq!(field, "test"),
        _ => panic!("Wrong variant"),
    }
}

#[test]
fn test_unnamed_fields() {
    #[derive(Constructor)]
    struct TestTuple(i32, String);

    let test = TestTuple::new(42, "test".to_string());
    assert_eq!(test.0, 42);
    assert_eq!(test.1, "test");
}

#[test]
fn test_ref() {
    use std::marker::PhantomData;

    #[derive(Constructor)]
    struct Test<'a, T> {
        field: &'a str,
        #[new(default)]
        phantom: PhantomData<T>,
    }

    let s = "test";
    let test: Test<'_, i32> = Test::new(s);
    assert_eq!(test.field, "test");
}

#[test]
fn test_unit_struct() {
    #[derive(Constructor)]
    struct Empty;

    let _empty = Empty::new();
}

#[test]
fn test_custom_field_value() {
    #[derive(Constructor, Debug, PartialEq)]
    struct Test {
        #[new(value = "42")]
        field1: i32,
        field2: String,
    }

    let test = Test::new("test".to_string());
    assert_eq!(test.field1, 42);
    assert_eq!(test.field2, "test");
}

#[test]
fn test_lint_attrs() {
    #[allow(dead_code, unused_variables)]
    #[derive(Constructor)]
    struct Test {
        field1: i32,
        field2: String,
    }

    let test = Test::new(42, "test".to_string());
    assert_eq!(test.field1, 42);
    assert_eq!(test.field2, "test");
}

#[test]
fn test_cfg_attr_lint() {
    #[cfg_attr(test, allow(dead_code))]
    #[derive(Constructor)]
    struct Test {
        field1: i32,
        field2: String,
    }

    let test = Test::new(42, "test".to_string());
    assert_eq!(test.field1, 42);
    assert_eq!(test.field2, "test");
}

#[test]
fn test_generic_with_bounds() {
    #[derive(Constructor)]
    struct Test<T: Clone + Default> {
        field: T,
        #[new(default)]
        extra: String,
    }

    let test = Test::new(42);
    assert_eq!(test.field, 42);
    assert_eq!(test.extra, String::default());
}

#[test]
fn test_multiple_field_attrs() {
    #[derive(Constructor)]
    struct Test {
        field1: i32,
        #[allow(dead_code)]
        #[new(default)]
        field2: String,
    }

    let test = Test::new(42);
    assert_eq!(test.field1, 42);
    assert_eq!(test.field2, String::default());
}

// The following tests verify compile-time errors

// This should fail to compile:
// #[test]
// fn test_empty_enum() {
//     #[derive(Constructor)]
//     enum Empty {}
// }

// This should fail to compile:
// #[test]
// fn test_enum_with_discriminant() {
//     #[derive(Constructor)]
//     enum Test {
//         A = 1,
//         B = 2,
//     }
// }

// This should fail to compile:
// #[test]
// fn test_multiple_new_attrs() {
//     #[derive(Constructor)]
//     struct Test {
//         #[new(default)]
//         #[new(value = "42")]
//         field: i32,
//     }
// }

// This should fail to compile:
// #[test]
// fn test_invalid_new_attr() {
//     #[derive(Constructor)]
//     struct Test {
//         #[new(invalid)]
//         field: i32,
//     }
// }
