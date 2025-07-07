/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalityGraphError;
use deep_causality_macros::Constructor;
use std::error::Error;
use std::fmt;
use ultragraph::GraphError;

#[derive(Constructor, Debug)]
pub struct CausalityError(pub String);

impl Error for CausalityError {}

impl fmt::Display for CausalityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CausalityError: {}", self.0)
    }
}

impl From<GraphError> for CausalityError {
    fn from(err: GraphError) -> Self {
        // Convert the specific graph error into a descriptive string
        // and wrap it in our custom error type.
        CausalityError(format!("Graph operation failed: {err}"))
    }
}

impl From<CausalityGraphError> for CausalityError {
    fn from(err: CausalityGraphError) -> Self {
        // Convert the graph-specific error into our general causality error.
        CausalityError(format!("Causal graph error: {err}"))
    }
}
