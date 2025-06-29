/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::prelude::{ModelGenerativeError, ModelValidationError};
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum ModelBuildError {
    GenerationFailed(String),
    ValidationFailed(String),
}

impl Error for ModelBuildError {}

impl Display for ModelBuildError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ModelBuildError::GenerationFailed(e) => {
                write!(f, "The generation process failed: {}", e)
            }
            ModelBuildError::ValidationFailed(e) => {
                write!(
                    f,
                    "The generative output was invalid for model construction: {}",
                    e
                )
            }
        }
    }
}

impl From<ModelGenerativeError> for ModelBuildError {
    fn from(value: ModelGenerativeError) -> Self {
        ModelBuildError::GenerationFailed(value.to_string())
    }
}

impl From<ModelValidationError> for ModelBuildError {
    fn from(value: ModelValidationError) -> Self {
        ModelBuildError::ValidationFailed(value.to_string())
    }
}
