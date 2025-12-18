/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::SimplicialComplex;
use deep_causality_tensor::CausalTensor;
use std::sync::Arc;

mod clone;
mod cup_product;
mod display;
mod getters;

/// Represents a discrete field defined on the k-skeleton.
/// (e.g., Temperature on Vertices, Magnetic Flux on Faces).
#[derive(Clone, Debug)]
pub struct Topology<T> {
    /// Shared reference to the underlying mesh
    pub(crate) complex: Arc<SimplicialComplex>,
    /// The dimension of the simplices this data lives on
    pub(crate) grade: usize,
    /// The values (CausalTensor is essentially a dense vector here)
    pub(crate) data: CausalTensor<T>,
    /// The Focus (Cursor) for Comonadic extraction
    pub(crate) cursor: usize,
}

use crate::TopologyError;

impl<T> Topology<T> {
    pub fn new(
        complex: Arc<SimplicialComplex>,
        grade: usize,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {
        // Validate grade
        if grade > complex.max_simplex_dimension() {
            return Err(TopologyError::InvalidGradeOperation(format!(
                "grade {} exceeds max dimension {}",
                grade,
                complex.max_simplex_dimension()
            )));
        }

        // Validate data/skeleton match
        // Note: skeleton might be empty if grade exists but no simplices?
        // complex.skeletons() returns a slice of skeletons.
        // We need to check if grade index is valid for skeletons vector first.
        if grade >= complex.skeletons.len() {
            return Err(TopologyError::InvalidInput(format!(
                "grade {} exceeds available skeletons {}",
                grade,
                complex.skeletons.len()
            )));
        }

        let expected_size = complex.skeletons[grade].simplices.len();
        if data.len() != expected_size {
            return Err(TopologyError::InvalidInput(format!(
                "data length {} does not match skeleton size {} for grade {}",
                data.len(),
                expected_size,
                grade
            )));
        }

        // Validate cursor bounds
        if cursor >= data.len() && !data.is_empty() {
            return Err(TopologyError::IndexOutOfBounds(format!(
                "cursor {} is out of bounds for data length {}",
                cursor,
                data.len()
            )));
        }

        Ok(Self {
            complex,
            grade,
            data,
            cursor,
        })
    }
}
