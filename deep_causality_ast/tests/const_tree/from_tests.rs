/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
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
