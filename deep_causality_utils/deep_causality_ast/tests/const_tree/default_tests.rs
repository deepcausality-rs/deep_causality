/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_default() {
    let tree: ConstTree<i32> = ConstTree::default();
    assert_eq!(*tree.value(), 0);
    assert!(tree.is_leaf());
}

#[test]
fn test_default_delegates_to_the_value_type() {
    // `i32::default()` is 0, which any hard-coded zero-like default would also produce. A type
    // whose default is not numeric shows the delegation.
    let text: ConstTree<String> = ConstTree::default();
    assert_eq!(text.value(), "");
    assert!(text.is_leaf());

    let flag: ConstTree<bool> = ConstTree::default();
    assert!(!*flag.value());
    assert!(flag.is_leaf());
}
