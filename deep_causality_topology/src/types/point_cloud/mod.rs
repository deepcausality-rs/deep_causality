/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use alloc::string::ToString;

use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;

use crate::TopologyError;

mod base_topology;
mod display;
mod getters;
mod op_triangulate;

/// Represents a collection of data points in a d-dimensional space.
/// This is a "0-Complex" that can be used to infer higher-order topological structures.
#[derive(Debug, Clone, PartialEq)]
pub struct PointCloud<T> {
    /// The coordinates of the points. Typically NxM where N is the number of points
    /// and M is the dimensionality of the space.
    pub(crate) points: CausalTensor<f64>,
    /// Optional metadata associated with each point.
    pub(crate) metadata: CausalTensor<T>,
    /// The Focus (Cursor) for Comonadic extraction
    pub(crate) cursor: usize,
}

impl<T> PointCloud<T> {
    /// Creates a new `PointCloud` from a `CausalTensor` of points and optional metadata.
    /// The `points` tensor is expected to have a shape suitable for N points in M dimensions (e.g., `[N, M]`).
    /// The `metadata` tensor is expected to have a shape suitable for N items of metadata (e.g., `[N]`).
    pub fn new(
        points: CausalTensor<f64>,
        metadata: CausalTensor<T>,
        cursor: usize,
    ) -> Result<Self, TopologyError>
    where
        T: Default + Copy + Clone + PartialEq + Zero,
    {
        if points.is_empty() || points.shape().is_empty() {
            return Err(TopologyError::InvalidInput(
                "PointCloud `points` cannot be empty or have invalid shape".to_string(),
            ));
        }
        if points.shape()[0] != metadata.shape()[0] {
            return Err(TopologyError::InvalidInput(
                "Number of points and metadata items must match".to_string(),
            ));
        }
        if cursor >= points.len() {
            return Err(TopologyError::IndexOutOfBounds(
                "Initial cursor out of bounds for PointCloud".to_string(),
            ));
        }

        // Additional validation for point dimension could be added here if needed,
        // e.g., points.shape()[1] should be consistent if a specific dimension is expected.
        Ok(Self {
            points,
            metadata,
            cursor,
        })
    }

    /// Returns the number of points in the cloud.
    pub fn len(&self) -> usize {
        self.points.shape()[0]
    }

    /// Returns true if the point cloud contains no points.
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Returns the current cursor position.
    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Creates a shallow clone of the PointCloud.
    pub fn clone_shallow(&self) -> Self
    where
        T: Clone,
    {
        PointCloud {
            points: self.points.clone(),
            metadata: self.metadata.clone(),
            cursor: 0, // Reset cursor for the shallow clone, or keep it if behavior needs it
        }
    }
}
