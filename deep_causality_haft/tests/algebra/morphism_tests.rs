/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{FnMorphism, Morphism};

fn double(x: i32) -> i32 {
    x * 2
}

#[test]
fn test_identity_returns_input_unchanged() {
    let id = <FnMorphism as Morphism<FnMorphism>>::identity::<i32>();
    assert_eq!(<FnMorphism as Morphism<FnMorphism>>::apply(&id, 42), 42);
}

#[test]
fn test_identity_is_generic_over_type() {
    let id = <FnMorphism as Morphism<FnMorphism>>::identity::<&str>();
    assert_eq!(
        <FnMorphism as Morphism<FnMorphism>>::apply(&id, "hello"),
        "hello"
    );
}

#[test]
fn test_apply_runs_the_arrow() {
    let f: fn(i32) -> i32 = double;
    assert_eq!(<FnMorphism as Morphism<FnMorphism>>::apply(&f, 21), 42);
    // Equivalent to calling the function pointer directly.
    assert_eq!(<FnMorphism as Morphism<FnMorphism>>::apply(&f, 21), f(21));
}

#[test]
fn test_apply_changes_type() {
    let to_len: fn(&str) -> usize = |s| s.len();
    assert_eq!(
        <FnMorphism as Morphism<FnMorphism>>::apply(&to_len, "abcd"),
        4
    );
}
