/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_ast::ConstTree;

#[test]
fn test_display() {
    let tree = ConstTree::with_children(1, vec![ConstTree::new(2)]);
    let display_str = format!("{}", tree);
    let expected_display = "1\n    2\n";
    assert_eq!(display_str, expected_display);
}
