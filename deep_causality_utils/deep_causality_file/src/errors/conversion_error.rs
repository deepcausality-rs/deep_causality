/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::error::Error;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, PartialEq, Clone)]
pub struct ConversionError(pub String);

impl ConversionError {
    pub fn new(msg: &str) -> ConversionError {
        ConversionError(msg.to_string())
    }
}

impl Error for ConversionError {}

impl From<&str> for ConversionError {
    fn from(value: &str) -> Self {
        ConversionError::new(value)
    }
}

impl From<String> for ConversionError {
    fn from(value: String) -> Self {
        ConversionError::new(value.as_str())
    }
}

impl Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ConversionError: {}", self.0)
    }
}
