/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use deep_causality_macros::Constructor;

#[derive(Constructor, Debug)]
pub struct UltraGraphError(pub String);

impl Error for UltraGraphError {}

impl fmt::Display for UltraGraphError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UltraGraphError: {}", self.0)
    }
}
