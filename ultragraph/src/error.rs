// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct HyperGraphError(pub String);

impl Error for HyperGraphError {}

impl fmt::Display for HyperGraphError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HyperGraphError: {}", self.0)
    }
}