/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use alloc::{string::String, string::ToString};
use core::error::Error;
use core::fmt;

/// A coordinate outside the range of its chart, or not finite: a latitude beyond ±90°.
#[derive(Debug)]
pub struct CoordinateError(pub String);

impl CoordinateError {
    pub fn new(field0: String) -> Self {
        Self(field0)
    }
}

impl Error for CoordinateError {}

impl fmt::Display for CoordinateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CoordinateError: {}", self.0)
    }
}

impl From<String> for CoordinateError {
    fn from(s: String) -> Self {
        CoordinateError(s)
    }
}

impl From<&str> for CoordinateError {
    fn from(s: &str) -> Self {
        CoordinateError(s.to_string())
    }
}
