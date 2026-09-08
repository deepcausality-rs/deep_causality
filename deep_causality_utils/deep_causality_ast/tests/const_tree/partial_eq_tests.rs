/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
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

#[test]
fn test_equality_recurses_past_the_immediate_children() {
    // Every other comparison in the crate is at most two levels deep, so a non-recursive `eq`
    // that compared the root value and the immediate children's values passed everything.
    let deep = |leaf: i32| {
        ConstTree::with_children(
            1,
            vec![ConstTree::with_children(2, vec![ConstTree::new(leaf)])],
        )
    };
    assert_eq!(deep(3), deep(3));
    assert_ne!(deep(3), deep(99));
}

#[test]
fn test_equality_is_sensitive_to_child_order_and_to_shape() {
    // Every other fixture has exactly one child, where order cannot matter. An `eq` comparing
    // children as a set, ignoring their order, treats these two as equal.
    let ab = ConstTree::with_children(0, vec![ConstTree::new(1), ConstTree::new(2)]);
    let ba = ConstTree::with_children(0, vec![ConstTree::new(2), ConstTree::new(1)]);
    assert_ne!(ab, ba);

    // The same multiset of values in a different shape: content alone does not decide equality.
    let nested = ConstTree::with_children(
        1,
        vec![ConstTree::with_children(2, vec![ConstTree::new(3)])],
    );
    let flat = ConstTree::with_children(1, vec![ConstTree::new(2), ConstTree::new(3)]);
    assert_ne!(nested, flat);

    // And a differing child count at equal prefixes.
    assert_ne!(ab, ConstTree::with_children(0, vec![ConstTree::new(1)]));
}

/// `Eq` is a marker with no methods, so only a bound can require it. Nothing in the suite did,
/// which means `impl<T: Eq> Eq for ConstTree<T>` could have been deleted unnoticed.
fn requires_total_equality<T: Eq>(a: &T, b: &T) -> bool {
    a == b
}

#[test]
fn test_the_tree_is_totally_equatable_when_its_values_are() {
    let a = ConstTree::with_children(1, vec![ConstTree::new(2)]);
    let b = ConstTree::with_children(1, vec![ConstTree::new(2)]);
    let c = ConstTree::with_children(1, vec![ConstTree::new(3)]);
    assert!(requires_total_equality(&a, &b));
    assert!(!requires_total_equality(&a, &c));
}
