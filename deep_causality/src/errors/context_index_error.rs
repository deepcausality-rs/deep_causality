// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::Constructor;
use std::error::Error;
use std::fmt;

#[derive(Constructor, Debug)]
pub struct ContextIndexError(pub String);

impl Error for ContextIndexError {}

impl fmt::Display for ContextIndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ContextIndexError: {}", self.0)
    }
}
