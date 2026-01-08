/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_get_child() {
    let leaf1 = ConstTree::new(1);
    let leaf2 = ConstTree::new(2);
    let tree = ConstTree::with_children(0, vec![leaf1.clone(), leaf2.clone()]);

    assert_eq!(tree.get_child(0), Some(&leaf1));
    assert_eq!(tree.get_child(1), Some(&leaf2));
    assert_eq!(tree.get_child(2), None);
}

#[test]
fn test_size() {
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::new(2),
            ConstTree::with_children(3, vec![ConstTree::new(4)]),
        ],
    );
    assert_eq!(tree.size(), 4);
    assert_eq!(ConstTree::new(0).size(), 1);
}

#[test]
fn test_depth() {
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::new(2),
            ConstTree::with_children(3, vec![ConstTree::new(4)]),
        ],
    );
    assert_eq!(tree.depth(), 3);
    let leaf = ConstTree::new(0);
    assert_eq!(leaf.depth(), 1);
    let empty_children_tree: ConstTree<i32> = ConstTree::with_children(0, vec![]);
    assert_eq!(empty_children_tree.depth(), 1);
}

#[test]
fn test_get_id() {
    let tree1 = ConstTree::new(10);
    let tree2 = ConstTree::new(10); // Same value, different allocation
    let tree1_clone = tree1.clone();

    // IDs should be non-zero memory addresses
    assert_ne!(tree1.get_id(), 0);
    assert_ne!(tree2.get_id(), 0);

    // A clone should have the same ID as the original
    assert_eq!(tree1.get_id(), tree1_clone.get_id());

    // Independently created trees should have different IDs
    assert_ne!(tree1.get_id(), tree2.get_id());

    // Verify that ptr_eq and get_id are consistent
    assert!(tree1.ptr_eq(&tree1_clone));
    assert_eq!(tree1.get_id(), tree1_clone.get_id());

    assert!(!tree1.ptr_eq(&tree2));
    assert_ne!(tree1.get_id(), tree2.get_id());
}

#[test]
fn test_search() {
    let tree = ConstTree::with_children(
        10,
        vec![
            ConstTree::new(20),
            ConstTree::with_children(30, vec![ConstTree::new(40)]),
        ],
    );

    // find
    let found = tree.find(|v| *v == 30).unwrap();
    assert_eq!(*found.value(), 30);
    assert!(!found.is_leaf());

    assert!(tree.find(|v| *v == 99).is_none());

    // find_all
    let all_gt_15: Vec<_> = tree.find_all(|v| *v > 15).map(|n| *n.value()).collect();
    assert_eq!(all_gt_15, vec![20, 30, 40]);

    // contains
    assert!(tree.contains(&20));
    assert!(!tree.contains(&99));
}
