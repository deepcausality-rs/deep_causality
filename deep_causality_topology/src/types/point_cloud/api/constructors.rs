/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Constructor API for PointCloud.

use crate::{PointCloud, TopologyError};
use deep_causality_num::Zero;
use deep_causality_tensor::CausalTensor;

impl<T> PointCloud<T> {
    /// Creates a new `PointCloud` from points and metadata tensors.
    ///
    /// # Arguments
    /// * `points` - Tensor of NxM (N points in M dimensions)
    /// * `metadata` - Tensor of N metadata items
    /// * `cursor` - Initial cursor position
    ///
    /// # Returns
    /// * `Ok(PointCloud)` - A valid point cloud
    /// * `Err(TopologyError)` - If validation fails
    pub fn new(
        points: CausalTensor<f64>,
        metadata: CausalTensor<T>,
        cursor: usize,
    ) -> Result<Self, TopologyError>
    where
        T: Default + Copy + Clone + PartialEq + Zero,
    {
        Self::new_cpu(points, metadata, cursor)
    }
}
