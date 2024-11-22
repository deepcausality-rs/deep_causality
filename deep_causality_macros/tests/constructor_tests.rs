// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

#[test]
fn test_derive_struct() {
    use deep_causality_macros::Constructor;

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
    use deep_causality_macros::Constructor;

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
    use deep_causality_macros::Constructor;

    #[derive(Constructor)]
    struct TestTuple(i32, String);

    let test = TestTuple::new(42, "test".to_string());
    assert_eq!(test.0, 42);
    assert_eq!(test.1, "test");
}

#[test]
fn test_ref() {
    use deep_causality_macros::Constructor;
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
