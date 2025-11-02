/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ProbabilisticType;
use crate::errors::UncertainError;
use crate::types::cache::SampledValue;

impl ProbabilisticType for bool {
    fn to_sampled_value(&self) -> SampledValue {
        SampledValue::Bool(*self)
    }

    fn from_sampled_value(value: SampledValue) -> Result<Self, UncertainError> {
        match value {
            SampledValue::Bool(b) => Ok(b),
            _ => Err(UncertainError::UnsupportedTypeError(
                "Expected bool SampledValue".to_string(),
            )),
        }
    }

    fn default_value() -> Self {
        false
    }
}
