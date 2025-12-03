/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::PointCloud;
use deep_causality_tensor::CausalTensor;

impl<T> PointCloud<T> {
    pub fn points(&self) -> &CausalTensor<f64> {
        &self.points
    }

    pub fn metadata(&self) -> &CausalTensor<T> {
        &self.metadata
    }
}
