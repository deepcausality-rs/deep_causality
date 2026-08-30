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

#[test]
fn test_as_ref() {
    // Non-`Copy` payloads, so the consuming `left` / `right` would move out of the original;
    // `as_ref` reaches the inner value by reference and leaves the original owned.
    let l: Either<String, i32> = Either::Left("hi".to_string());
    let r: Either<String, i32> = Either::Right(7);

    assert_eq!(l.as_ref().left(), Some(&"hi".to_string()));
    assert_eq!(l.as_ref().right(), None);
    assert_eq!(r.as_ref().right(), Some(&7));
    assert_eq!(r.as_ref().left(), None);

    // Still owned and usable afterwards (as_ref did not consume them).
    assert!(l.is_left());
    assert!(r.is_right());
}

#[test]
fn test_as_mut() {
    let mut l: Either<String, i32> = Either::Left("hi".to_string());
    if let Either::Left(s) = l.as_mut() {
        s.push_str(" there");
    }
    assert_eq!(l, Either::Left("hi there".to_string()));

    let mut r: Either<String, i32> = Either::Right(7);
    if let Either::Right(n) = r.as_mut() {
        *n += 1;
    }
    assert_eq!(r, Either::Right(8));
}
