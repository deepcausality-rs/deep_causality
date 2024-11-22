// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::error::Error;
use std::fmt;
use deep_causality_macros::Constructor;

#[derive(Constructor, Debug)]
pub struct UpdateError(pub String);

impl Error for UpdateError {}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UpdateError: {}", self.0)
    }
}
