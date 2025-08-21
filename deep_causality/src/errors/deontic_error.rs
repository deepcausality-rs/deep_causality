/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TeloidID;
use std::error::Error;
use std::fmt;
use ultragraph::GraphError;

/// Errors related to the Deontic Inference process within the Effect Ethos.
#[derive(Debug, Clone, PartialEq)]
pub enum DeonticError {
    /// An operation was attempted on a graph that was not frozen.
    /// Deontic inference requires a static, immutable graph.
    GraphNotFrozen,

    /// The TeloidGraph is invalid because it contains a cycle, which would
    /// lead to infinite loops in reasoning.
    GraphIsCyclic,

    /// A TeloidID exists in the graph or tag index but not in the TeloidStore,
    /// indicating a critical state inconsistency.
    TeloidNotFound { id: TeloidID },

    /// The final set of active norms was empty or otherwise unable to produce
    /// a conclusive verdict.
    InconclusiveVerdict,

    /// Wraps a lower-level error from the ultragraph crate.
    GraphError(GraphError),
}

impl Error for DeonticError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DeonticError::GraphError(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for DeonticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeonticError::GraphNotFrozen => {
                write!(
                    f,
                    "Deontic inference failed: The TeloidGraph must be frozen before evaluation."
                )
            }
            DeonticError::GraphIsCyclic => {
                write!(
                    f,
                    "Deontic inference failed: The TeloidGraph contains a cycle and is invalid."
                )
            }
            DeonticError::TeloidNotFound { id } => {
                write!(
                    f,
                    "Deontic inference failed: Teloid with ID {} not found in store.",
                    id
                )
            }
            DeonticError::InconclusiveVerdict => {
                write!(
                    f,
                    "Deontic inference failed: The final set of active norms was inconclusive."
                )
            }
            DeonticError::GraphError(e) => {
                write!(
                    f,
                    "A graph operation failed during deontic inference: {}",
                    e
                )
            }
        }
    }
}

impl From<GraphError> for DeonticError {
    fn from(err: GraphError) -> Self {
        DeonticError::GraphError(err)
    }
}
