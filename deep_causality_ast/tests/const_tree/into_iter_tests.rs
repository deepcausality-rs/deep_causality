/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_consuming_iterator() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2), ConstTree::new(3)]);
    let tree_clone = tree.clone(); // Clone to prove original is moved.

    // into_iter consumes the tree.
    let values: Vec<_> = tree.into_iter().collect();
    assert_eq!(values, vec![1, 2, 3]);

    // The original `tree` variable is now moved and cannot be used.
    // assert_eq!(*tree.value(), 1); // This line would fail to compile.

    // The clone is still valid.
    assert_eq!(*tree_clone.value(), 1);
}

#[test]
fn test_consuming_iterator_ok_branch() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2), ConstTree::new(3)]);

    // Call into_iter on the original tree, ensuring no other clones exist.
    // This should trigger the Ok branch of Arc::try_unwrap for the root node.
    let values: Vec<_> = tree.into_iter().collect();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn test_consuming_iterator_shared_arc() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2), ConstTree::new(3)]);
    let tree_clone1 = tree.clone();
    let tree_clone2 = tree.clone(); // Ensure multiple references

    // Call into_iter on one of the clones. This should trigger the Err branch.
    let values: Vec<_> = tree_clone1.into_iter().collect();
    assert_eq!(values, vec![1, 2, 3]);

    // Verify that the other clone is still valid.
    assert_eq!(*tree_clone2.value(), 1);
    assert_eq!(tree_clone2.children().len(), 2);
}
