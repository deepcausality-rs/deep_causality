// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use ultragraph::prelude::UltraGraphError;

#[test]
fn test_ultra_graph_error() {
    let x = 1;
    let result: Result<usize, UltraGraphError> = Err(UltraGraphError(
        format!("unexpected number {}", x),
    ));
    let build_error = result.unwrap_err();
    assert_eq!(build_error.to_string(), format!("UltraGraphError: unexpected number {}", 1));
}