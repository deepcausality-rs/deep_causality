/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Exercises the *default* method bodies of the `Adjustable<T>` and
//! `UncertainAdjustable` traits.
//!
//! The default implementations intentionally do nothing and simply return
//! `Ok(())` so that `update`/`adjust` are optional for node types. To cover
//! those default bodies we declare two minimal types that implement the traits
//! without overriding any method, then invoke the inherited defaults.

use deep_causality::utils_test::test_utils_array_grid;
use deep_causality::{Adjustable, UncertainAdjustable};

/// A trivial type relying entirely on the default `Adjustable<i32>` impl.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct PlainNode {
    value: i32,
}

impl Adjustable<i32> for PlainNode {}

/// A trivial type relying entirely on the default `UncertainAdjustable` impl.
#[derive(Debug, Default, Clone, PartialEq)]
struct PlainUncertainNode;

impl UncertainAdjustable for PlainUncertainNode {
    type Data = f64;
}

#[test]
fn test_adjustable_default_update_is_noop_ok() {
    let mut node = PlainNode { value: 7 };
    let grid = test_utils_array_grid::get_1d_array_grid(99);

    // The default body returns Ok(()) and leaves the node untouched.
    let res = node.update(&grid);
    assert!(res.is_ok());
    assert_eq!(node.value, 7);
}

#[test]
fn test_adjustable_default_adjust_is_noop_ok() {
    let mut node = PlainNode { value: 3 };
    let grid = test_utils_array_grid::get_1d_array_grid(123);

    let res = node.adjust(&grid);
    assert!(res.is_ok());
    assert_eq!(node.value, 3);
}

#[test]
fn test_uncertain_adjustable_default_update_is_ok() {
    let mut node = PlainUncertainNode;
    let res = node.update(1.5_f64);
    assert!(res.is_ok());
}

#[test]
fn test_uncertain_adjustable_default_adjust_is_ok() {
    let mut node = PlainUncertainNode;
    let res = node.adjust(-2.5_f64);
    assert!(res.is_ok());
}
