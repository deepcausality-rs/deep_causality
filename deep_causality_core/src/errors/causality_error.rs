/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
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

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CausalityErrorEnum {
    // Generic Errors
    #[default]
    Unspecified = 0,
    InternalLogicError = 1,  // For logic paths that should be unreachable
    TypeConversionError = 2, // For failures in `FromProtocol`

    // Graph Execution Errors
    StartNodeOutOfBounds = 10,
    MaxStepsExceeded = 11,
    GraphExecutionProducedNoResult = 12,
    // Add other specific errors as they are identified
}
