/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_rand::RngError;

#[test]
fn test_os_random_generator_error() {
    let error_msg = "Failed to get random bytes from OS";
    let error = RngError::OsRandomGenerator(error_msg.to_string());
    assert_eq!(
        format!("{}", error),
        format!("OS random generator error: {}", error_msg)
    );
}
