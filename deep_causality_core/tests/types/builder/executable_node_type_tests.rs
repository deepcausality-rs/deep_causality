/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::NodeType;

#[test]
fn test_node_type_new_and_id() {
    let node_type: NodeType<i32, i32> = NodeType::new(1);
    assert_eq!(node_type.id(), 1);
}

#[test]
fn test_node_type_clone_copy() {
    let node_type: NodeType<i32, i32> = NodeType::new(1);
    let node_type_clone = node_type; // Copy
    let node_type_clone2 = node_type; // Clone

    assert_eq!(node_type.id(), node_type_clone.id());
    assert_eq!(node_type.id(), node_type_clone2.id());
}
