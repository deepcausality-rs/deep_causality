/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::prelude::{CausaloidId, ContextId, ContextoidId};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Represents specific validation errors that can occur during model construction
/// from a generative output.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum ModelValidationError {
    MissingCreateCausaloid,
    DuplicateCausaloidID { id: CausaloidId },
    DuplicateContextId { id: ContextId },
    DuplicateContextoidId { id: ContextoidId },
    TargetContextNotFound { id: ContextId },
    TargetCausaloidNotFound { id: CausaloidId },
    TargetContextoidNotFound { id: CausaloidId },
    UnsupportedOperation { operation: String },
}

impl Error for ModelValidationError {}

impl Display for ModelValidationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelValidationError::MissingCreateCausaloid => {
                write!(
                    f,
                    "The generative output is missing the mandatory Causaloid creation command."
                )
            }
            ModelValidationError::DuplicateCausaloidID { id } => {
                write!(
                    f,
                    "The generative output contains more than one 'CreateCausaloid' command, but exactly one is required. Duplicate ID: {id}"
                )
            }
            ModelValidationError::DuplicateContextId { id } => {
                write!(
                    f,
                    "The generative output contains a 'CreateContext' command with a duplicate ID: {id}"
                )
            }
            ModelValidationError::DuplicateContextoidId { id } => {
                write!(
                    f,
                    "The generative output contains a 'CreateContextoid' command with a duplicate ID: {id}"
                )
            }
            ModelValidationError::TargetContextNotFound { id } => {
                write!(
                    f,
                    "The generative output attempts to add a Contextoid to a Context (ID: {id}) that was not created in the same generative step."
                )
            }

            ModelValidationError::TargetCausaloidNotFound { id } => {
                write!(
                    f,
                    "The generative output attempts to add a Causaloid (ID: {id})
                    that was not created in the same generative step."
                )
            }

            ModelValidationError::TargetContextoidNotFound { id } => {
                write!(
                    f,
                    "The generative output attempts to add a Contextoid {id}) that was not created in the same generative step."
                )
            }

            ModelValidationError::UnsupportedOperation { operation } => {
                write!(
                    f,
                    "An unsupported operation was used during model construction: {operation}. Only creation commands are allowed."
                )
            }
        }
    }
}
