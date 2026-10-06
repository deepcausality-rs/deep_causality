/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils::inertia::inertia;
use crate::{MetricTensor4D, MetricTensorError, TangentSpacetime};
use deep_causality_algebra::RealField;

impl<R: RealField> MetricTensor4D for TangentSpacetime<R> {
    fn metric_tensor(&self) -> [[R; 4]; 4] {
        self.metric
    }

    /// Replaces the tensor with `new_metric` when every entry is finite and it is symmetric and of
    /// signature (−, +, +, +): one negative and three positive eigenvalues, none zero.
    ///
    /// # Errors
    /// [`MetricTensorError`] naming the first rule broken, checked in that order, leaving the
    /// tensor unchanged.
    fn update_metric_tensor(&mut self, new_metric: [[R; 4]; 4]) -> Result<(), MetricTensorError> {
        let entries = (0..4).flat_map(|i| (0..4).map(move |j| (i, j)));
        if let Some((i, j)) = entries
            .clone()
            .find(|&(i, j)| !new_metric[i][j].is_finite())
        {
            return Err(MetricTensorError::NonFinite(i, j));
        }
        if let Some((i, j)) = entries
            .filter(|&(i, j)| i < j)
            .find(|&(i, j)| new_metric[i][j] != new_metric[j][i])
        {
            return Err(MetricTensorError::Asymmetric(i, j));
        }
        let (positive, negative, zero) = inertia(&new_metric);
        if (positive, negative, zero) != (3, 1, 0) {
            return Err(MetricTensorError::Signature(positive, negative, zero));
        }
        self.metric = new_metric;
        Ok(())
    }
}
