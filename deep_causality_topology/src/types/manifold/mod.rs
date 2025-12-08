/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{ReggeGeometry, SimplicialComplex, TopologyError};
use core::fmt;
use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;
use std::fmt::Formatter;

mod base_topology;
mod differential;
mod geometry;
mod manifold_topology;
mod simplicial_topology;
mod utils;

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
        if !utils::is_oriented(complex) {
            return false;
        }

        // A manifold must satisfy the link condition
        if !utils::satisfies_link_condition(complex) {
            return false;
        }

        // All checks passed
        true
    }
}

impl<T> Manifold<T>
where
    T: Clone,
{
    /// Creates a shallow clone of the Manifold.
    pub fn clone_shallow(&self) -> Self {
        Manifold {
            complex: self.complex.clone(),
            data: self.data.clone(),
            metric: self.metric.clone(),
            cursor: 0,
        }
    }
}

impl<T> fmt::Display for Manifold<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Manifold {{ dimension: {}, simplices: {} }}",
            self.complex.skeletons.last().map(|s| s.dim).unwrap_or(0),
            self.complex
                .skeletons
                .iter()
                .map(|s| s.simplices.len())
                .sum::<usize>()
        )
    }
}

impl<T> Manifold<T> {
    pub fn complex(&self) -> &SimplicialComplex {
        &self.complex
    }

    pub fn data(&self) -> &CausalTensor<T> {
        &self.data
    }

    pub fn metric(&self) -> Option<&ReggeGeometry> {
        self.metric.as_ref()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }
}
