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
    /// Failed to add a new teloid to the graph.
    FailedToAddTeloid,

    /// Failed to add edge to the graph.
    FailedToAddEdge(usize, usize, GraphError),

    /// An operation was attempted on a graph that was not frozen.
    /// Deontic inference requires a static, immutable graph.
    GraphNotFrozen(GraphError),

    /// An mutation operation was attempted on a graph that was frozen.
    /// Mutation requires an unfrozen graph
    GraphIsFrozen(GraphError),

    /// The TeloidGraph is invalid because it contains a cycle, which would
    /// lead to infinite loops in reasoning.
    GraphIsCyclic(GraphError),

    /// A TeloidID exists in the graph or tag index but not in the TeloidStore,
    /// indicating a critical state inconsistency.
    TeloidNotFound { id: TeloidID },

    /// The final set of active norms was empty or otherwise unable to produce
    /// a conclusive verdict.
    InconclusiveVerdict,

    /// No applicable norms were found for a given action.
    NoRelevantNormsFound,

    /// The CausalState is missing a context, which is required for deontic evaluation.
    MissingContext,

    /// Wraps a lower-level error from the ultragraph crate.
    GraphError(GraphError),
}

impl Error for DeonticError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DeonticError::GraphError(e) => Some(e),
            DeonticError::FailedToAddEdge(_, _, e) => Some(e),
            DeonticError::GraphNotFrozen(e) => Some(e),
            DeonticError::GraphIsFrozen(e) => Some(e),
            DeonticError::GraphIsCyclic(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for DeonticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeonticError::GraphNotFrozen(_) => {
                write!(
                    f,
                    "Deontic inference failed: The TeloidGraph must be frozen before evaluation."
                )
            }
            DeonticError::GraphIsFrozen(_) => {
                write!(
                    f,
                    "Deontic inference failed: The TeloidGraph is frozen and cannot be modified."
                )
            }

            DeonticError::GraphIsCyclic(_) => {
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
            DeonticError::MissingContext => {
                write!(
                    f,
                    "Deontic inference failed: The CausalState is missing a context."
                )
            }
            DeonticError::GraphError(e) => {
                write!(
                    f,
                    "A graph operation failed during deontic inference: {}",
                    e
                )
            }
            DeonticError::FailedToAddTeloid => {
                write!(f, "Failed to add a new teloid to the graph.")
            }
            DeonticError::FailedToAddEdge(source, target, _) => {
                write!(
                    f,
                    "Edge from {source} to {target} could not be created; a node may not exist or the edge already exists."
                )
            }
            DeonticError::NoRelevantNormsFound => {
                write!(
                    f,
                    "No relevant norms found, so the action cannot be decided. Please check if you have added the correct tags."
                )
            }
        }
    }
}

impl From<GraphError> for DeonticError {
    fn from(err: GraphError) -> Self {
        match err {
            GraphError::GraphIsFrozen => DeonticError::GraphIsFrozen(err),
            GraphError::GraphNotFrozen => DeonticError::GraphNotFrozen(err),
            GraphError::GraphContainsCycle => DeonticError::GraphIsCyclic(err),
            GraphError::EdgeCreationError { source, target } => {
                DeonticError::FailedToAddEdge(source, target, err)
            }
            _ => DeonticError::GraphError(err),
        }
    }
}
