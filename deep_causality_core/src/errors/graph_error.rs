/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    StartNodeOutOfBounds(usize),
    MaxStepsExceeded(usize),
    GraphExecutionProducedNoResult,
}

impl Display for GraphError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            GraphError::StartNodeOutOfBounds(id) => {
                write!(f, "Start node index {} out of bounds", id)
            }
            GraphError::MaxStepsExceeded(steps) => {
                write!(f, "Execution exceeded max_steps limit of {}", steps)
            }

            GraphError::GraphExecutionProducedNoResult => {
                write!(f, "Graph execution produced no result")
            }
        }
    }
}
