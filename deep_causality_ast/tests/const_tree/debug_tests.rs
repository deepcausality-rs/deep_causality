/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_debug() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2)]);
    let debug_str = format!("{:?}", tree);
    let expected_debug = "ConstTree { value: 1, children_count: 1 }";
    assert_eq!(debug_str, expected_debug);
}
