/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;
use std::sync::Arc;
use std::thread;

#[test]
fn test_new_leaf() {
    let leaf = ConstTree::new(10);
    assert_eq!(*leaf.value(), 10);
    assert!(leaf.is_leaf());
    assert_eq!(leaf.children().len(), 0);
}

#[test]
fn test_with_children() {
    let leaf1 = ConstTree::new(1);
    let leaf2 = ConstTree::new(2);
    let tree = ConstTree::with_children(0, vec![leaf1.clone(), leaf2.clone()]);

    assert_eq!(*tree.value(), 0);
    assert!(!tree.is_leaf());
    assert_eq!(tree.children().len(), 2);
    assert_eq!(tree.children()[0], leaf1);
    assert_eq!(tree.children()[1], leaf2);
}

#[test]
fn test_with_children_accepts_any_by_value_iterator() {
    // The documented forms. A slice is deliberately absent: `&[ConstTree<T>]` yields references,
    // so it does not satisfy `IntoIterator<Item = Self>` and the doc used to claim it did.
    let from_vec = ConstTree::with_children(0, vec![ConstTree::new(1), ConstTree::new(2)]);
    let from_array = ConstTree::with_children(0, [ConstTree::new(1), ConstTree::new(2)]);
    let from_iter = ConstTree::with_children(0, (1..3).map(ConstTree::new));
    assert_eq!(from_vec, from_array);
    assert_eq!(from_vec, from_iter);
    assert_eq!(from_vec.children().len(), 2);

    // An empty iterator makes a leaf, not a node with an empty child.
    let empty: ConstTree<i32> = ConstTree::with_children(0, Vec::new());
    assert!(empty.is_leaf());
}

#[test]
fn test_with_value_shares_the_children_rather_than_copying_them() {
    // `with_value` is documented O(1) precisely because the children are shared. Comparing the
    // two child slices by value cannot see the difference: a deep copy compares equal.
    let original = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(3)]),
            ConstTree::new(4),
        ],
    );
    let renamed = original.with_value(99);

    assert_eq!(*original.value(), 1);
    assert_eq!(*renamed.value(), 99);
    assert_eq!(original.children().len(), renamed.children().len());
    for (a, b) in original.children().iter().zip(renamed.children()) {
        assert!(a.ptr_eq(b), "child subtree was copied, not shared");
    }
}

#[test]
fn test_thread_safety() {
    let tree = Arc::new(ConstTree::with_children(1, vec![ConstTree::new(2)]));
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let tree_clone = Arc::clone(&tree);
            thread::spawn(move || {
                // Each thread works with its clone
                let modified = tree_clone.add_child(ConstTree::new(100));
                assert_eq!(modified.children().len(), 2);
                assert_eq!(*modified.children()[1].value(), 100);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Original tree is unchanged
    assert_eq!(tree.children().len(), 1);
}
