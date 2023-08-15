// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct UltraGraphError(pub String);

impl Error for UltraGraphError {}

impl fmt::Display for UltraGraphError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UltraGraphError: {}", self.0)
    }
}
