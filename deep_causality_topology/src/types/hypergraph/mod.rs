/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TopologyError;
use deep_causality_num::Zero;
use deep_causality_sparse::CsrMatrix;
use deep_causality_tensor::CausalTensor;

mod base_topology;
mod clone;
mod display;
mod getters;
mod graph_topology;
mod hypergraph_topology;

/// Represents a hypergraph where hyperedges can connect an arbitrary number of nodes.
/// The incidence matrix efficiently stores the relationships.
/// The type parameter T represents metadata associated with each node.
#[derive(Debug, Clone, PartialEq)]
pub struct Hypergraph<T> {
    /// Number of nodes in the hypergraph.
    pub(crate) num_nodes: usize,
    /// Number of hyperedges in the hypergraph.
    pub(crate) num_hyperedges: usize,
    /// Incidence matrix: rows represent nodes, columns represent hyperedges.
    /// An entry (i, j) is 1 if node i is part of hyperedge j, 0 otherwise.
    /// CsrMatrix<i8> is suitable for sparse incidence matrices.
    pub(crate) incidence: CsrMatrix<i8>,
    /// Metadata associated with each node
    pub(crate) data: CausalTensor<T>,
    /// The Focus (Cursor) for Comonadic extraction
    pub(crate) cursor: usize,
}

impl<T> Hypergraph<T>
where
    T: Default + Copy + Clone + PartialEq + Zero,
{
    /// Creates a new `Hypergraph` from an incidence matrix and node metadata.
    /// The matrix should have dimensions `num_nodes` x `num_hyperedges`.
    pub fn new(
        incidence: CsrMatrix<i8>,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {
        let (num_nodes, num_hyperedges) = incidence.shape();
        if num_nodes == 0 || num_hyperedges == 0 {
            return Err(TopologyError::InvalidInput(
                "Hypergraph must have at least one node and one hyperedge".to_string(),
            ));
        }

        // Validate data size matches num_nodes
        if data.len() != num_nodes {
            return Err(TopologyError::InvalidInput(
                "Data size must match number of nodes".to_string(),
            ));
        }

        if cursor >= num_nodes {
            return Err(TopologyError::IndexOutOfBounds(
                "Initial cursor out of bounds for Hypergraph".to_string(),
            ));
        }

        // Validate incidence matrix values (should be 0 or 1 for standard hypergraphs)
        for &val in incidence.values() {
            if val != 0 && val != 1 {
                return Err(TopologyError::HypergraphError(
                    "Incidence matrix values must be 0 or 1".to_string(),
                ));
            }
        }

        Ok(Self {
            num_nodes,
            num_hyperedges,
            incidence,
            data,
            cursor,
        })
    }

    /// Returns the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }
}
