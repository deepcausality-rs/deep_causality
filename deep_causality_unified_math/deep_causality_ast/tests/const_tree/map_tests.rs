/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_map() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2), ConstTree::new(3)]);
    let mapped_tree = tree.map(&mut |v| v * 2);

    assert_eq!(*mapped_tree.value(), 2);
    assert_eq!(*mapped_tree.children()[0].value(), 4);
    assert_eq!(*mapped_tree.children()[1].value(), 6);
    // Original is unchanged
    assert_eq!(*tree.value(), 1);
}

#[test]
fn test_into_map() {
    let tree = ConstTree::with_children(10, vec![ConstTree::new(20)]);
    let tree_clone = tree.clone();

    // into_map consumes the tree.
    let mapped_tree = tree.into_map(|v| v.to_string());

    assert_eq!(*mapped_tree.value(), "10");
    assert_eq!(*mapped_tree.children()[0].value(), "20");

    // Original `tree` is moved.
    // The clone is still valid and unchanged.
    assert_eq!(*tree_clone.value(), 10);
    assert_eq!(*tree_clone.children()[0].value(), 20);
}

#[test]
fn test_join() {
    // Create a tree of trees: ConstTree<ConstTree<i32>>
    // Structure:
    //   Inner1(1)
    //   - Inner2(2)
    //     - Leaf(3)
    //   - Leaf(4)
    let leaf3 = ConstTree::new(3);
    let inner2 = ConstTree::with_children(2, vec![leaf3]);
    let leaf4 = ConstTree::new(4);
    let inner1 = ConstTree::with_children(1, vec![inner2, leaf4]);

    // Wrap them in an outer tree
    let tree_of_trees = ConstTree::new(inner1);

    // Join the tree
    let joined_tree = tree_of_trees.join();

    // Expected structure after join:
    //   1
    //   - 2
    //     - 3
    //   - 4
    assert_eq!(*joined_tree.value(), 1);
    assert_eq!(joined_tree.children().len(), 2);
    assert_eq!(*joined_tree.children()[0].value(), 2);
    assert_eq!(*joined_tree.children()[1].value(), 4);
    assert_eq!(joined_tree.children()[0].children().len(), 1);
    assert_eq!(*joined_tree.children()[0].children()[0].value(), 3);
    assert!(joined_tree.children()[1].is_leaf());

    let expected_vals: Vec<_> = joined_tree.iter_pre_order().copied().collect();
    assert_eq!(expected_vals, vec![1, 2, 3, 4]);
}
