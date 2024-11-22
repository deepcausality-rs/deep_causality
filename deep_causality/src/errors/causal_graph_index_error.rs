// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::Constructor;
use std::error::Error;
use std::fmt;

#[derive(Constructor, Debug)]
pub struct CausalGraphIndexError(pub String);

impl Error for CausalGraphIndexError {}

impl fmt::Display for CausalGraphIndexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CausalGraphIndexError: {}", self.0)
    }
}
