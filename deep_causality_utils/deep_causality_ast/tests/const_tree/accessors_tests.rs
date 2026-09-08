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

    // The mirror image, with the deep branch first. With it only ever last, taking the maximum
    // over all children is indistinguishable from following the last child alone.
    let deep_branch_first = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(4)]),
            ConstTree::new(3),
        ],
    );
    assert_eq!(deep_branch_first.depth(), 3);
    assert_eq!(deep_branch_first.size(), 4);
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
    // The deep branch comes first, so pre-order (10, 20, 40, 30) and level-order (10, 20, 30, 40)
    // disagree. On the flat-then-deep shape they coincide, and the documented pre-order of `find`
    // and `find_all` cannot be told from breadth-first.
    let tree = ConstTree::with_children(
        10,
        vec![
            ConstTree::with_children(20, vec![ConstTree::new(40)]),
            ConstTree::new(30),
        ],
    );

    // find searches the whole tree, root and grandchildren included.
    let found = tree.find(|v| *v == 20).unwrap();
    assert_eq!(*found.value(), 20);
    assert!(!found.is_leaf());
    assert_eq!(*tree.find(|v| *v == 10).unwrap().value(), 10);
    assert_eq!(*tree.find(|v| *v == 40).unwrap().value(), 40);
    assert!(tree.find(|v| *v == 99).is_none());

    // With several matches it returns the first in pre-order, not the last or the shallowest.
    assert_eq!(*tree.find(|v| *v > 15).unwrap().value(), 20);

    // find_all yields every match in pre-order; level-order would give [20, 30, 40].
    let all_gt_15: Vec<_> = tree.find_all(|v| *v > 15).map(|n| *n.value()).collect();
    assert_eq!(all_gt_15, vec![20, 40, 30]);

    // contains asks the same question of the whole tree, including the root itself.
    assert!(tree.contains(&10));
    assert!(tree.contains(&20));
    assert!(tree.contains(&40));
    assert!(!tree.contains(&99));
}
