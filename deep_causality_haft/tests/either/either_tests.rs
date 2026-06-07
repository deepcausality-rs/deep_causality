/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::Either;

#[test]
fn test_is_left() {
    let l: Either<i32, &str> = Either::Left(1);
    let r: Either<i32, &str> = Either::Right("x");
    assert!(l.is_left());
    assert!(!r.is_left());
}

#[test]
fn test_is_right() {
    let l: Either<i32, &str> = Either::Left(1);
    let r: Either<i32, &str> = Either::Right("x");
    assert!(!l.is_right());
    assert!(r.is_right());
}

#[test]
fn test_left_accessor() {
    let l: Either<i32, &str> = Either::Left(7);
    let r: Either<i32, &str> = Either::Right("x");
    assert_eq!(l.left(), Some(7));
    assert_eq!(r.left(), None);
}

#[test]
fn test_right_accessor() {
    let l: Either<i32, &str> = Either::Left(7);
    let r: Either<i32, &str> = Either::Right("x");
    assert_eq!(l.right(), None);
    assert_eq!(r.right(), Some("x"));
}

#[test]
fn test_equality_and_clone() {
    let a: Either<i32, &str> = Either::Left(1);
    let b = a;
    assert_eq!(a, b);
    assert_eq!(a.clone(), Either::Left(1));
    assert_ne!(a, Either::Left(2));
    assert_ne!(a, Either::Right("1"));
}

#[test]
fn test_debug_and_hash() {
    use std::collections::HashSet;

    let l: Either<i32, i32> = Either::Left(1);
    let r: Either<i32, i32> = Either::Right(1);
    // Debug
    assert_eq!(format!("{l:?}"), "Left(1)");
    assert_eq!(format!("{r:?}"), "Right(1)");
    // Hash / Eq: Left(1) and Right(1) are distinct keys.
    let set: HashSet<Either<i32, i32>> = [l, r].into_iter().collect();
    assert_eq!(set.len(), 2);
}
