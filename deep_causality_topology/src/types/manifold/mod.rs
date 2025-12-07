/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{ReggeGeometry, SimplicialComplex, TopologyError};
use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;

mod base_topology;
mod clone;
mod differential;
mod display;
mod geometry;
mod getters;
mod manifold_checks;
mod manifold_topology;
mod simplicial_topology;

/// A newtype wrapper around `SimplicialComplex` that represents a Manifold.
/// Its construction enforces geometric properties essential for physics simulations.
/// The type parameter T represents data living on the manifold's simplices.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifold<T> {
    /// The underlying simplicial complex, guaranteed to satisfy manifold properties.
    pub(crate) complex: SimplicialComplex,
    /// The data associated with the manifold (e.g., scalar field values on simplices)
    pub(crate) data: CausalTensor<T>,
    /// The metric information of the manifold, containing edge lengths.
    pub(crate) metric: Option<ReggeGeometry>,
    /// The Focus (Cursor) for Comonadic extraction
    pub(crate) cursor: usize,
}

impl<T> Manifold<T>
where
    T: Default + Copy + Clone + PartialEq + Zero,
{
    /// Attempts to create a new `Manifold` from a `SimplicialComplex` and data.
    /// This constructor performs rigorous checks to ensure the complex satisfies manifold criteria.
    ///
    /// # Errors
    /// Returns `Err(TopologyError::ManifoldError)` if the input `SimplicialComplex`
    /// does not meet the requirements to be classified as a manifold.
    pub fn new(
        complex: SimplicialComplex,
        data: CausalTensor<T>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {
        // Validation: Check data size matches complex
        let expected_size = complex.skeletons.iter().map(|s| s.simplices.len()).sum();
        if data.len() != expected_size {
            return Err(TopologyError::InvalidInput(
                "Data size must match total number of simplices in complex".to_string(),
            ));
        }

        if cursor >= data.len() {
            return Err(TopologyError::IndexOutOfBounds(
                "Initial cursor out of bounds for Manifold".to_string(),
            ));
        }

        if !Self::check_is_manifold(&complex) {
            return Err(TopologyError::ManifoldError(
                "SimplicialComplex does not satisfy manifold properties".to_string(),
            ));
        }

        Ok(Self {
            complex,
            data,
            metric: None,
            cursor,
        })
    }

    pub fn with_metric(
        complex: SimplicialComplex,
        data: CausalTensor<T>,
        metric: Option<ReggeGeometry>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {
        // Validation: Check data size matches complex
        let expected_size = complex.skeletons.iter().map(|s| s.simplices.len()).sum();
        if data.len() != expected_size {
            return Err(TopologyError::InvalidInput(
                "Data size must match total number of simplices in complex".to_string(),
            ));
        }

        if let Some(ref regge) = metric {
            if let Some(skeleton_1) = complex.skeletons.get(1) {
                if skeleton_1.simplices.len() != regge.edge_lengths.len() {
                    return Err(TopologyError::InvalidInput(
                        "Metric edge_lengths size must match number of 1-simplices".to_string(),
                    ));
                }
            } else if !regge.edge_lengths.is_empty() {
                return Err(TopologyError::InvalidInput(
                    "Metric provided but complex has no 1-simplices".to_string(),
                ));
            }
        }

        if cursor >= data.len() {
            return Err(TopologyError::IndexOutOfBounds(
                "Initial cursor out of bounds for Manifold".to_string(),
            ));
        }

        if !Self::check_is_manifold(&complex) {
            return Err(TopologyError::ManifoldError(
                "SimplicialComplex does not satisfy manifold properties".to_string(),
            ));
        }

        Ok(Self {
            complex,
            data,
            metric,
            cursor,
        })
    }

    /// Internal helper function to determine if a `SimplicialComplex` is a manifold.
    /// Uses the ManifoldTopology trait methods to perform validation.
    fn check_is_manifold(complex: &SimplicialComplex) -> bool {
        // Basic check: complex must have at least one skeleton
        if complex.skeletons.is_empty() {
            return false;
        }

        // For a proper manifold, we need non-trivial structure
        // At minimum, need vertices (0-skeleton)
        let num_vertices = complex
            .skeletons
            .first()
            .map(|s| s.simplices.len())
            .unwrap_or(0);
        if num_vertices == 0 {
            return false;
        }

        // A manifold must be oriented
        if !manifold_checks::is_oriented(complex) {
            return false;
        }

        // A manifold must satisfy the link condition
        if !manifold_checks::satisfies_link_condition(complex) {
            return false;
        }

        // All checks passed
        true
    }
}
