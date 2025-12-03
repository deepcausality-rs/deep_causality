/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::ExecutableNode;

#[test]
#[cfg(not(feature = "strict-zst"))]
fn test_executable_node_new_and_getters() {
    let id = 1;
    let func = Box::new(|x: i32| x + 1);
    let node = ExecutableNode::new(id, func);

    assert_eq!(node.id(), 1);

    // Test execution
    let result = (node.func())(10);
    assert_eq!(result, 11);
}

#[test]
fn test_executable_node_display() {
    #[cfg(not(feature = "strict-zst"))]
    let node = ExecutableNode::new(1, Box::new(|x: i32| x));

    #[cfg(feature = "strict-zst")]
    let node = ExecutableNode::new(1, |x: i32| x);

    assert_eq!(format!("{}", node), "ExecutableNode(id: 1)");
}
