/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ProbabilisticType;
use crate::errors::UncertainError;
use crate::types::cache::SampledValue;

impl ProbabilisticType for f64 {
    fn to_sampled_value(&self) -> SampledValue {
        SampledValue::Float(*self)
    }

    fn from_sampled_value(value: SampledValue) -> Result<Self, UncertainError> {
        match value {
            SampledValue::Float(f) => Ok(f),
            _ => Err(UncertainError::UnsupportedTypeError(
                "Expected f64 SampledValue".to_string(),
            )),
        }
    }

    fn default_value() -> Self {
        0.0
    }
}
