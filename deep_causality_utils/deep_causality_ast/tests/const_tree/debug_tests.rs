/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_debug() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2)]);
    let debug_str = format!("{:?}", tree);
    let expected_debug = "ConstTree { value: 1, children_count: 1 }";
    assert_eq!(debug_str, expected_debug);
}

#[test]
fn test_debug_reports_direct_children_not_descendants() {
    // On `1[2]` the direct-child count, the descendant count, `size() - 1` and `depth() - 1` are
    // all 1, so the asserted string cannot tell them apart. Here they are 3, 4, 4 and 2.
    let tree = ConstTree::with_children(
        1,
        vec![
            ConstTree::with_children(2, vec![ConstTree::new(3)]),
            ConstTree::new(4),
            ConstTree::new(5),
        ],
    );
    assert_eq!(tree.children().len(), 3);
    assert_eq!(tree.size(), 5);
    assert_eq!(tree.depth(), 3);
    assert_eq!(
        format!("{tree:?}"),
        "ConstTree { value: 1, children_count: 3 }"
    );

    // A leaf reports zero, which no non-empty count can produce.
    assert_eq!(
        format!("{:?}", ConstTree::new(9)),
        "ConstTree { value: 9, children_count: 0 }"
    );
}
