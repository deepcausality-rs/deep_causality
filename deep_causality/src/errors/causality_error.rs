/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_macros::Constructor;
use std::error::Error;
use std::fmt;

#[derive(Constructor, Debug)]
pub struct CausalityError(pub String);

impl Error for CausalityError {}

impl fmt::Display for CausalityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CausalityError: {}", self.0)
    }
}
