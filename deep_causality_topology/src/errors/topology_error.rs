/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use core::fmt;
use deep_causality_multivector::CausalMultiVectorError;
use deep_causality_tensor::CausalTensorError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TopologyError {
    /// Error indicating a simplex was not found in the complex or skeleton.
    SimplexNotFound,
    /// Error indicating a dimension mismatch in operations.
    DimensionMismatch(String),
    /// Error indicating an invalid operation for a given grade (e.g., boundary of 0-chain).
    InvalidGradeOperation(String),
    /// Error indicating an index is out of bounds for a data structure.
    IndexOutOfBounds(String),
    /// Error during tensor creation or manipulation.
    TensorError(String),
    /// Error specific to PointCloud operations.
    PointCloudError(String),
    /// Error specific to Graph operations.
    GraphError(String),
    /// Error specific to Hypergraph operations.
    HypergraphError(String),
    /// Error specific to Manifold operations or validation.
    ManifoldError(String),
    /// Error during multivector operations.
    MultivectorError(String),
    /// Error indicating a malformed or invalid input structure.
    InvalidInput(String),
    /// General catch-all error for other topological issues.
    GenericError(String),
}

impl fmt::Display for TopologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopologyError::SimplexNotFound => write!(f, "Simplex not found"),
            TopologyError::DimensionMismatch(msg) => write!(f, "Dimension mismatch: {}", msg),
            TopologyError::InvalidGradeOperation(msg) => {
                write!(f, "Invalid grade operation: {}", msg)
            }
            TopologyError::IndexOutOfBounds(msg) => write!(f, "Index out of bounds: {}", msg),
            TopologyError::TensorError(msg) => write!(f, "Tensor error: {}", msg),
            TopologyError::PointCloudError(msg) => write!(f, "PointCloud error: {}", msg),
            TopologyError::GraphError(msg) => write!(f, "Graph error: {}", msg),
            TopologyError::HypergraphError(msg) => write!(f, "Hypergraph error: {}", msg),
            TopologyError::ManifoldError(msg) => write!(f, "Manifold error: {}", msg),
            TopologyError::MultivectorError(msg) => write!(f, "Multivector error: {}", msg),
            TopologyError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            TopologyError::GenericError(msg) => write!(f, "Topology error: {}", msg),
        }
    }
}

impl std::error::Error for TopologyError {}

impl From<CausalTensorError> for TopologyError {
    fn from(err: CausalTensorError) -> Self {
        TopologyError::TensorError(err.to_string())
    }
}

impl From<CausalMultiVectorError> for TopologyError {
    fn from(err: CausalMultiVectorError) -> Self {
        TopologyError::MultivectorError(err.to_string())
    }
}
