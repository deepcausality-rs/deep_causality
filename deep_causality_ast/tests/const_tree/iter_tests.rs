/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_iterators() {
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(3)]),
            ConstTree::new(4),
        ],
    );

    // Pre-order
    let pre_order_vals: Vec<_> = tree.iter_pre_order().copied().collect();
    assert_eq!(pre_order_vals, vec![1, 2, 3, 4]);

    // Post-order
    let post_order_vals: Vec<_> = tree.iter_post_order().copied().collect();
    assert_eq!(post_order_vals, vec![3, 2, 4, 1]);

    // Level-order
    let level_order_vals: Vec<_> = tree.iter_level_order().copied().collect();
    assert_eq!(level_order_vals, vec![1, 2, 4, 3]);
}

#[test]
fn test_node_iterator() {
    let child1 = ConstTree::new(2);
    let child2 = ConstTree::new(3);
    let tree = ConstTree::with_children(1, vec![child1.clone(), child2.clone()]);

    let nodes: Vec<_> = tree.iter_nodes_pre_order().collect();

    assert_eq!(nodes.len(), 3);
    assert!(nodes[0].ptr_eq(&tree));
    assert!(nodes[1].ptr_eq(&child1));
    assert!(nodes[2].ptr_eq(&child2));
}
