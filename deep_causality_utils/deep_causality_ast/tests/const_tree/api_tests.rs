/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

fn child_values(tree: &ConstTree<i32>) -> Vec<i32> {
    tree.children().iter().map(|c| *c.value()).collect()
}

#[test]
fn test_modification_methods() {
    // Three distinct children. With one, index 0 is the only in-bounds index, so neither
    // `update_child` nor `remove_child` is ever observed to use its `index` argument: bodies
    // hard-coded to 0, or that clear the whole child list, pass a one-child fixture.
    let original = ConstTree::with_children(
        10,
        vec![ConstTree::new(11), ConstTree::new(12), ConstTree::new(13)],
    );

    // with_value: a new root value over the same children.
    let modified_value = original.with_value(20);
    assert_eq!(*original.value(), 10);
    assert_eq!(*modified_value.value(), 20);
    assert_eq!(original.children(), modified_value.children()); // Children are shared

    // add_child appends, leaving the existing children in place and in order.
    let added_child = original.add_child(ConstTree::new(14));
    assert_eq!(child_values(&original), vec![11, 12, 13]);
    assert_eq!(child_values(&added_child), vec![11, 12, 13, 14]);

    // replace_children discards all of them.
    let replaced_children = original.replace_children(vec![ConstTree::new(100)]);
    assert_eq!(child_values(&original), vec![11, 12, 13]);
    assert_eq!(child_values(&replaced_children), vec![100]);
    assert_ne!(original.children(), replaced_children.children());

    // update_child replaces the child AT `index` and leaves its siblings untouched.
    let updated_child = original.update_child(1, ConstTree::new(99)).unwrap();
    assert_eq!(child_values(&original), vec![11, 12, 13]);
    assert_eq!(child_values(&updated_child), vec![11, 99, 13]);
    assert_eq!(
        child_values(&original.update_child(0, ConstTree::new(99)).unwrap()),
        vec![99, 12, 13]
    );
    assert_eq!(
        child_values(&original.update_child(2, ConstTree::new(99)).unwrap()),
        vec![11, 12, 99]
    );
    // The bound is `index >= len`, so `len` itself is the first rejected index.
    assert!(original.update_child(3, ConstTree::new(999)).is_none());

    // remove_child removes the child AT `index` and keeps the remainder in order.
    assert_eq!(
        child_values(&original.remove_child(0).unwrap()),
        vec![12, 13]
    );
    assert_eq!(
        child_values(&original.remove_child(1).unwrap()),
        vec![11, 13]
    );
    assert_eq!(
        child_values(&original.remove_child(2).unwrap()),
        vec![11, 12]
    );
    assert_eq!(child_values(&original), vec![11, 12, 13]);
    assert!(original.remove_child(3).is_none());
}
