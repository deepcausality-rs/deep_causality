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
