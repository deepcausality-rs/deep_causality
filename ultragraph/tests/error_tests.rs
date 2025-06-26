/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use ultragraph::prelude::UltraGraphError;

#[test]
fn test_ultra_graph_error() {
    let x = 1;
    let result: Result<usize, UltraGraphError> =
        Err(UltraGraphError::new(format!("unexpected number {}", x)));
    assert!(result.is_err(),);
}
