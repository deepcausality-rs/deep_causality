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
