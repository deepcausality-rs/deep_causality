/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_equality() {
    let tree1 = ConstTree::with_children(1, vec![ConstTree::new(2)]);
    let tree2 = ConstTree::with_children(1, vec![ConstTree::new(2)]);
    let tree3 = ConstTree::with_children(1, vec![ConstTree::new(99)]);
    let leaf = ConstTree::new(1);

    // Deep equality
    assert_eq!(tree1, tree2);
    assert_ne!(tree1, tree3);
    assert_ne!(tree1, leaf);

    // Pointer equality
    let tree1_clone = tree1.clone();
    assert_eq!(tree1, tree1_clone);
    // Check that the internal Arcs point to the same allocation
    assert!(tree1.ptr_eq(&tree1_clone));
    // tree2 has the same structure but is a different allocation
    assert!(!tree1.ptr_eq(&tree2));
}
