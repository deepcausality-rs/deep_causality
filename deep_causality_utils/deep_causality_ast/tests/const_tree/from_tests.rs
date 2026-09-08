/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_from_value() {
    let tree = ConstTree::from(42);
    assert_eq!(*tree.value(), 42);
    assert!(tree.is_leaf());
}

#[test]
fn test_from_ref_value() {
    let value = 100;
    let tree: ConstTree<i32> = ConstTree::from(&value);
    assert_eq!(*tree.value(), 100);
    assert!(tree.is_leaf());
}

#[test]
fn test_from_ref_clones_a_non_copy_value() {
    // With `i32` the borrow and the clone are indistinguishable. A `String` shows that the
    // conversion copies the value rather than depending on the borrow outliving the tree.
    let owned = String::from("root");
    let tree: ConstTree<String> = ConstTree::from(&owned);
    assert_eq!(tree.value(), "root");
    assert!(tree.is_leaf());
    // The source is untouched and independent of the tree.
    assert_eq!(owned, "root");
    drop(owned);
    assert_eq!(tree.value(), "root");
}
