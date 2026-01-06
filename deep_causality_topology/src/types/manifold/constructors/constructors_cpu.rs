/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! CPU implementation of Manifold constructors.

use crate::{ReggeGeometry, SimplicialComplex, TopologyError};
use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;

use super::super::{Manifold, utils};

impl<C, D> Manifold<C, D>
where
    C: Default + Copy + Clone + PartialEq + Zero,
    D: Default + Copy + Clone + PartialEq + Zero,
{
    /// CPU implementation of Manifold constructor.
    pub(crate) fn new_cpu(
        complex: SimplicialComplex<C>,
        data: CausalTensor<D>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {
        // Validation: Check data size matches complex
        let expected_size: usize = complex.skeletons.iter().map(|s| s.simplices.len()).sum();
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

        if !Self::check_is_manifold_cpu(&complex) {
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

    /// CPU implementation of Manifold constructor with metric.
    pub(crate) fn with_metric_cpu(
        complex: SimplicialComplex<C>,
        data: CausalTensor<D>,
        metric: Option<ReggeGeometry<C>>,
        cursor: usize,
    ) -> Result<Self, TopologyError> {
        // Validation: Check data size matches complex
        let expected_size: usize = complex.skeletons.iter().map(|s| s.simplices.len()).sum();
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

        if !Self::check_is_manifold_cpu(&complex) {
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

    /// CPU implementation: check if complex satisfies manifold properties.
    fn check_is_manifold_cpu(complex: &SimplicialComplex<C>) -> bool {
        // Basic check: complex must have at least one skeleton
        if complex.skeletons.is_empty() {
            return false;
        }

        // For a proper manifold, we need non-trivial structure
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

        true
    }
}

impl<C, D> Manifold<C, D>
where
    C: Clone,
    D: Clone,
{
    /// CPU implementation of shallow clone.
    pub(crate) fn clone_shallow_cpu(manifold: &Self) -> Self {
        Manifold {
            complex: manifold.complex.clone(),
            data: manifold.data.clone(),
            metric: manifold.metric.clone(),
            cursor: 0,
        }
    }
}
