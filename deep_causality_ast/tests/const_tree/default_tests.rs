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
