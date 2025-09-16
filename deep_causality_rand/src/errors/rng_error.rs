/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::error::Error;

#[derive(Debug)]
pub enum RngError {
    OsRandomGenerator(String),
    // Add other error variants as needed
}

impl Error for RngError {}

impl std::fmt::Display for RngError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RngError::OsRandomGenerator(e) => write!(f, "OS random generator error: {}", e),
            // Format other error variants as needed
        }
    }
}
