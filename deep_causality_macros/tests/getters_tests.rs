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
        #[getter(name = get_value)] // Rename getter methods as you wish
        field: i32,
    }

    let test = Test { field: 42 };
    assert_eq!(test.get_value(), &42);
}

#[test]
fn test_visibility() {
    mod inner {
        use deep_causality_macros::Getters;

        #[derive(Getters)]
        pub struct Test {
            field1: i32,
            field2: i32,
        }

        pub fn create_test() -> Test {
            Test {
                field1: 42,
                field2: 43,
            }
        }
    }

    let test = inner::create_test();
    assert_eq!(test.field1(), &42);
    assert_eq!(test.field2(), &43);
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
fn test_multiple_attributes() {
    #[derive(Getters)]
    struct Test {
        #[getter(name = get_value)]
        field: i32,
    }

    let test = Test { field: 42 };
    assert_eq!(test.get_value(), &42);
}
