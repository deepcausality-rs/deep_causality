/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CausalityError;
use deep_causality_macros::Constructor;
use std::error::Error;
use std::fmt;
use ultragraph::GraphError;

#[derive(Constructor, Debug)]
pub struct CausalityGraphError(pub String);

impl Error for CausalityGraphError {}

impl fmt::Display for CausalityGraphError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CausalityGraphError: {}", self.0)
    }
}

/// This implementation allows for the automatic conversion of a `GraphError`
/// from the `ultragraph` crate into a `CausalityGraphError`. This is essential
/// for using the `?` operator to propagate errors cleanly.
impl From<GraphError> for CausalityGraphError {
    fn from(err: GraphError) -> Self {
        CausalityGraphError(err.to_string())
    }
}

impl From<CausalityError> for CausalityGraphError {
    fn from(err: CausalityError) -> Self {
        CausalityGraphError(err.to_string())
    }
}
