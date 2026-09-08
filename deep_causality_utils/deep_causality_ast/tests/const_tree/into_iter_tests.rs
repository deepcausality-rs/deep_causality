/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The consuming iterator. Every fixture here is `1 -> [2 -> [3], 4]`, on which pre-order
//! (1, 2, 3, 4), level-order (1, 2, 4, 3) and post-order (3, 2, 4, 1) all differ. On the flat
//! `1 -> [2, 3]` the first two coincide and the documented pre-order is unpinned, in both the
//! owned and the shared arm of `Arc::try_unwrap`.

use deep_causality_ast::ConstTree;

#[test]
fn test_consuming_iterator() {
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(3)]),
            ConstTree::new(4),
        ],
    );
    let tree_clone = tree.clone(); // Clone to prove original is moved.

    // into_iter consumes the tree.
    let values: Vec<_> = tree.into_iter().collect();
    assert_eq!(values, vec![1, 2, 3, 4]);

    // The original `tree` variable is now moved and cannot be used.
    // assert_eq!(*tree.value(), 1); // This line would fail to compile.

    // The clone is still valid.
    assert_eq!(*tree_clone.value(), 1);
}

#[test]
fn test_consuming_iterator_ok_branch() {
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(3)]),
            ConstTree::new(4),
        ],
    );

    // Call into_iter on the original tree, ensuring no other clones exist.
    // This should trigger the Ok branch of Arc::try_unwrap for the root node.
    let values: Vec<_> = tree.into_iter().collect();
    assert_eq!(values, vec![1, 2, 3, 4]);
}

#[test]
fn test_consuming_iterator_shared_arc() {
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(3)]),
            ConstTree::new(4),
        ],
    );
    let tree_clone1 = tree.clone();
    let tree_clone2 = tree.clone(); // Ensure multiple references

    // Call into_iter on one of the clones. This should trigger the Err branch.
    let values: Vec<_> = tree_clone1.into_iter().collect();
    assert_eq!(values, vec![1, 2, 3, 4]);

    // Verify that the other clone is still valid.
    assert_eq!(*tree_clone2.value(), 1);
    assert_eq!(tree_clone2.children().len(), 2);
}

#[test]
fn test_consuming_a_leaf_yields_one_value_then_stops() {
    // The degenerate input: the stack empties after the first `next`, so the `?` on `pop`
    // is what ends the iteration rather than an exhausted child list.
    let mut it = ConstTree::new(7).into_iter();
    assert_eq!(it.next(), Some(7));
    assert_eq!(it.next(), None);
    assert_eq!(it.next(), None);

    // And the `for` sugar reaches the same iterator.
    let mut seen = Vec::new();
    for v in ConstTree::with_children(1, vec![ConstTree::new(2)]) {
        seen.push(v);
    }
    assert_eq!(seen, vec![1, 2]);
}
