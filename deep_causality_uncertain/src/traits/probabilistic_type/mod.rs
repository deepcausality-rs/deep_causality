/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
mod bool_probabilistic_type;
mod f64_probabilistic_type;

use crate::errors::UncertainError;
use crate::types::cache::SampledValue;

pub trait ProbabilisticType {
    /// Converts the type into an internal `SampledValue` representation.
    fn to_sampled_value(&self) -> SampledValue;
    /// Attempts to convert an internal `SampledValue` back into this type.
    fn from_sampled_value(value: SampledValue) -> Result<Self, UncertainError>
    where
        Self: Sized;
    /// Provides a default or zero-equivalent value for the type.
    fn default_value() -> Self;
}
