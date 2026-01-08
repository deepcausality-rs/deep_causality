/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Bifunctor, Promonad};
use deep_causality_haft::{Tuple2Witness, Tuple3Witness};

#[test]
fn test_tuple2_bimap() {
    let t = (10, "hello");
    let res = Tuple2Witness::bimap(t, |x| x * 2, |s| s.len());
    assert_eq!(res, (20, 5));
}

#[test]
fn test_tuple2_first() {
    let t = (10, "hello");
    let res = Tuple2Witness::first(t, |x| x * 2);
    assert_eq!(res, (20, "hello"));
}

#[test]
fn test_tuple2_second() {
    let t = (10, "hello");
    let res = Tuple2Witness::second(t, |s| s.len());
    assert_eq!(res, (10, 5));
}

#[test]
fn test_tuple3_merge() {
    let t1 = (1, 2, 3);
    let t2 = (10, 20, 30);
    let res = Tuple3Witness::merge(t1, t2, |a, b| a + b);
    assert_eq!(res, (11, 22, 33));
}

#[test]
#[should_panic(expected = "Tuple3Witness::fuse is not supported")]
fn test_tuple3_fuse_panic() {
    let _: (i32, i32, i32) = Tuple3Witness::fuse(1, 2);
}
