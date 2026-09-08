/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_clone_is_cheap() {
    let original = ConstTree::new(10);
    let cloned = original.clone();
    assert!(original.ptr_eq(&cloned));
    assert_eq!(*original.value(), *cloned.value());
}

#[test]
fn test_clone_shares_the_whole_subtree() {
    // The only clone fixture was a leaf, where "shares its children" is vacuously true. An
    // implementation that rebuilt the root and dropped or copied the children passes that.
    let original = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(3)]),
            ConstTree::new(4),
        ],
    );
    let cloned = original.clone();

    assert!(original.ptr_eq(&cloned));
    assert_eq!(original, cloned);
    assert_eq!(cloned.size(), 4);
    assert_eq!(cloned.depth(), 3);
    // Sharing reaches all the way down, not just the root handle.
    assert!(original.children()[0].children()[0].ptr_eq(&cloned.children()[0].children()[0]));
}
