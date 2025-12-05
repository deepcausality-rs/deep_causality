/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use alloc::string::String;
use core::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct CausalityError(pub CausalityErrorEnum);

impl CausalityError {
    pub fn new(error_enum: CausalityErrorEnum) -> Self {
        CausalityError(error_enum)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CausalityError {}

impl Display for CausalityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        // Delegate to the debug representation of the inner enum.
        write!(f, "{:?}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum CausalityErrorEnum {
    // Generic Errors
    #[default]
    Unspecified,
    InternalLogicError,  // For logic paths that should be unreachable
    TypeConversionError, // For failures in `FromProtocol`
    ValueNotAvailable,   // For when a value is expected but None is found

    // Graph Execution Errors
    StartNodeOutOfBounds,
    MaxStepsExceeded,
    GraphExecutionProducedNoResult,

    // Migration / Legacy Support Errors
    Custom(String),
    ActionError(String),
    DeonticError(String),
    ModelError(String),
}
