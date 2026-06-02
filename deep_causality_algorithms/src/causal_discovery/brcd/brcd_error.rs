/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The single error type for the BRCD estimator.
//!
//! [`BrcdError`] is a newtype wrapper around [`BrcdErrorEnum`]: the public error
//! *type* is fixed (every BRCD function returns `Result<_, BrcdError>`), while the
//! set of failure *cases* can grow by adding variants to the enum as the
//! implementation matures — without changing any signature. This mirrors the
//! repository idiom (`TopologyError(TopologyErrorEnum)`).

use std::error::Error;
use std::fmt;

/// The error type returned by every fallible BRCD operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrcdError(pub BrcdErrorEnum);

/// The growable set of BRCD failure cases.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrcdErrorEnum {
    /// No observations were supplied.
    EmptyData,
    /// Input dimensions disagree — a row, label, or parent count/width does not
    /// match the rest of the inputs.
    DimensionMismatch,
    /// The graph carries an edge that is neither a directed arc nor an undirected
    /// edge (a bidirected or partially-oriented endpoint); equivalence-class
    /// enumeration is defined only for CPDAGs.
    NotACpdag,
    /// The directed-arc projection contains a cycle, so the input is not a valid
    /// CPDAG / DAG.
    NotAcyclic,
    /// The Markov-equivalence class exceeds the enumeration bound; enumeration is
    /// refused rather than truncated.
    ClassTooLarge {
        /// The bound that was exceeded.
        bound: usize,
    },
    /// A Newton / IRLS iteration produced a non-finite parameter (a degenerate
    /// design that regularization did not rescue).
    SingularSystem,
    /// A transform was applied to data outside its domain (e.g. `log` of a
    /// non-positive value) after the auto-downgrade ladder.
    InvalidTransformDomain,
    /// The Yeo-Johnson transform is not yet implemented (deferred; design D7).
    YeojohnsonUnsupported,
    /// The discrete node cardinality `K` is zero.
    ZeroCardinality,
    /// A discrete node value is `≥ K` (outside `0..K`).
    StateOutOfRange,
    /// A node index is outside the graph's vertex range.
    NodeOutOfBounds,
    /// The set of undirected edges incident on the candidate set is too large to
    /// enumerate (`2^edges` configurations exceeds the bound).
    ConfigSpaceTooLarge {
        /// The number of incident undirected edges that exceeded the bound.
        edges: usize,
    },
}

impl fmt::Display for BrcdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BRCD error: {}", self.0)
    }
}

impl fmt::Display for BrcdErrorEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrcdErrorEnum::EmptyData => write!(f, "no observations were supplied"),
            BrcdErrorEnum::DimensionMismatch => {
                write!(f, "input dimensions disagree (row/label/parent counts)")
            }
            BrcdErrorEnum::NotACpdag => {
                write!(f, "graph is not a CPDAG (a bidirected or circle endpoint)")
            }
            BrcdErrorEnum::NotAcyclic => write!(f, "directed-arc projection contains a cycle"),
            BrcdErrorEnum::ClassTooLarge { bound } => {
                write!(f, "equivalence class exceeds the enumeration bound {bound}")
            }
            BrcdErrorEnum::SingularSystem => {
                write!(f, "Newton/IRLS produced a non-finite parameter")
            }
            BrcdErrorEnum::InvalidTransformDomain => {
                write!(f, "value is outside the transform's domain")
            }
            BrcdErrorEnum::YeojohnsonUnsupported => {
                write!(f, "the Yeo-Johnson transform is not yet implemented")
            }
            BrcdErrorEnum::ZeroCardinality => write!(f, "discrete node cardinality K is zero"),
            BrcdErrorEnum::StateOutOfRange => write!(f, "a discrete node value is outside 0..K"),
            BrcdErrorEnum::NodeOutOfBounds => write!(f, "a node index is outside the graph"),
            BrcdErrorEnum::ConfigSpaceTooLarge { edges } => write!(
                f,
                "too many incident undirected edges to enumerate ({edges} edges)"
            ),
        }
    }
}

impl Error for BrcdError {}

impl BrcdError {
    /// Returns the underlying error case.
    pub fn kind(&self) -> &BrcdErrorEnum {
        &self.0
    }
}
