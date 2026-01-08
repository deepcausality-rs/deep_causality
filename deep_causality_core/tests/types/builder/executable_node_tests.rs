/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_core::ExecutableNode;

#[test]
fn test_executable_node_display() {
    #[cfg(not(feature = "strict-zst"))]
    let node = ExecutableNode::new(1, Box::new(|x: i32| x));

    #[cfg(feature = "strict-zst")]
    let node = ExecutableNode::new(1, |x: i32| x);

    assert_eq!(format!("{}", node), "ExecutableNode(id: 1)");
}
