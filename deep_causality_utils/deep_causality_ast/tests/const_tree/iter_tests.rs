/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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
    // The same asymmetric shape as above. On a flat `1 -> [2, 3]` the node iterator's descent is
    // never taken and its pre-order is indistinguishable from level-order, which matters because
    // `find` and `find_all` document pre-order and are built on this iterator.
    let grandchild = ConstTree::new(3);
    let child1 = ConstTree::with_children(2, vec![grandchild.clone()]);
    let child2 = ConstTree::new(4);
    let tree = ConstTree::with_children(1, vec![child1.clone(), child2.clone()]);

    let nodes: Vec<_> = tree.iter_nodes_pre_order().collect();

    assert_eq!(nodes.len(), 4);
    assert!(nodes[0].ptr_eq(&tree));
    assert!(nodes[1].ptr_eq(&child1));
    assert!(nodes[2].ptr_eq(&grandchild));
    assert!(nodes[3].ptr_eq(&child2));
}

#[test]
fn test_every_iterator_yields_a_leaf_exactly_once() {
    // The degenerate input each traversal must still terminate on: post-order pops its only
    // stack entry, and level-order drains its queue.
    let leaf = ConstTree::new(7);
    assert_eq!(leaf.iter_pre_order().copied().collect::<Vec<_>>(), vec![7]);
    assert_eq!(leaf.iter_post_order().copied().collect::<Vec<_>>(), vec![7]);
    assert_eq!(
        leaf.iter_level_order().copied().collect::<Vec<_>>(),
        vec![7]
    );
    assert_eq!(leaf.iter_nodes_pre_order().count(), 1);
}

#[test]
fn test_sibling_order_is_forward_at_every_level() {
    // Three siblings, so a reversal and a rotation are distinguishable; with two they are the
    // same permutation. The deep branch is in the middle, which no other fixture arranges.
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::new(2),
            ConstTree::with_children(3, vec![ConstTree::new(6), ConstTree::new(7)]),
            ConstTree::new(4),
        ],
    );
    assert_eq!(
        tree.iter_pre_order().copied().collect::<Vec<_>>(),
        vec![1, 2, 3, 6, 7, 4]
    );
    assert_eq!(
        tree.iter_post_order().copied().collect::<Vec<_>>(),
        vec![2, 6, 7, 3, 4, 1]
    );
    assert_eq!(
        tree.iter_level_order().copied().collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 6, 7]
    );
}
