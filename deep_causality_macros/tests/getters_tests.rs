// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::Getters;

#[test]
fn test_basic_getters() {
    #[derive(Getters)]
    struct Test {
        field1: i32,
        field2: String,
    }

    let test = Test {
        field1: 42,
        field2: "test".to_string(),
    };

    assert_eq!(test.field1(), &42);
    assert_eq!(test.field2(), &"test".to_string());
}

#[test]
fn test_custom_getter_name() {
    #[derive(Getters)]
    struct Test {
        #[getter(name = get_value)]
        field: i32,
    }

    let test = Test { field: 42 };
    assert_eq!(test.get_value(), &42);
}

#[test]
fn test_generic_types() {
    #[derive(Getters)]
    struct Test<T> {
        field: T,
    }

    let test = Test { field: 42 };
    assert_eq!(test.field(), &42);

    let test = Test {
        field: "test".to_string(),
    };
    assert_eq!(test.field(), &"test".to_string());
}

#[test]
fn test_empty_struct() {
    #[derive(Getters)]
    struct Empty {}

    let _empty = Empty {};
    // Should compile without errors
}

#[test]
fn test_multiple_custom_names() {
    #[derive(Getters)]
    struct Test {
        #[getter(name = get_first)]
        first: i32,
        #[getter(name = get_second)]
        second: String,
        third: bool, // default name
    }

    let test = Test {
        first: 42,
        second: "test".to_string(),
        third: true,
    };

    assert_eq!(test.get_first(), &42);
    assert_eq!(test.get_second(), &"test".to_string());
    assert_eq!(test.third(), &true);
}

#[test]
fn test_generic_with_bounds() {
    #[derive(Getters)]
    struct Test<T: Clone + Default> {
        field: T,
    }

    let test = Test { field: 42 };
    assert_eq!(test.field(), &42);
}

// The following tests verify compile-time errors

// This should fail to compile:
// #[test]
// fn test_derive_on_enum() {
//     #[derive(Getters)]
//     enum Test {
//         Variant1(i32),
//         Variant2 { field: String },
//     }
// }

// This should fail to compile:
// #[test]
// fn test_unnamed_fields() {
//     #[derive(Getters)]
//     struct Test(i32, String);
// }

// This should fail to compile:
// #[test]
// fn test_redundant_attributes() {
//     #[derive(Getters)]
//     struct Test {
//         #[getter(name = value1)]
//         #[getter(name = value2)]
//         field: i32,
//     }
// }

// This should fail to compile:
// #[test]
// fn test_invalid_attribute_syntax() {
//     #[derive(Getters)]
//     struct Test {
//         #[getter(invalid)]
//         field: i32,
//     }
// }
