/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents a sampled value that can be either a floating-point number or a boolean.
pub enum SampledValue {
    /// A floating-point sampled value.
    Float(f64),
    /// A boolean sampled value.
    Bool(bool),
}

impl Display for SampledValue {
    /// Formats the `SampledValue` for display.
    ///
    /// This implementation writes the underlying float or boolean value directly.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write into.
    ///
    /// # Returns
    ///
    /// A `std::fmt::Result` indicating success or failure of the formatting operation.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SampledValue::Float(value) => write!(f, "{}", value),
            SampledValue::Bool(value) => write!(f, "{}", value),
        }
    }
}
