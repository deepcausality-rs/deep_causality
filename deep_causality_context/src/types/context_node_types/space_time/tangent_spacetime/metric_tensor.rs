/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::utils::inertia::inertia;
use crate::{MetricTensor4D, TangentSpacetime, UpdateError};
use deep_causality_algebra::RealField;

impl<R: RealField> MetricTensor4D for TangentSpacetime<R> {
    fn metric_tensor(&self) -> [[R; 4]; 4] {
        self.metric
    }

    /// Replaces the tensor with `new_metric` when every entry is finite and it is symmetric and of
    /// signature (−, +, +, +): one negative and three positive eigenvalues, none zero.
    ///
    /// # Errors
    /// [`UpdateError`] naming the rule broken, leaving the tensor unchanged.
    fn update_metric_tensor(&mut self, new_metric: [[R; 4]; 4]) -> Result<(), UpdateError> {
        if new_metric.iter().flatten().any(|v| !v.is_finite()) {
            return Err(UpdateError(
                "metric tensor has an entry that is not finite".into(),
            ));
        }
        let asymmetric = (0..4)
            .flat_map(|i| ((i + 1)..4).map(move |j| (i, j)))
            .find(|&(i, j)| new_metric[i][j] != new_metric[j][i]);
        if let Some((i, j)) = asymmetric {
            return Err(UpdateError(alloc::format!(
                "metric tensor is not symmetric: g[{i}][{j}] differs from g[{j}][{i}]"
            )));
        }
        let (positive, negative, zero) = inertia(&new_metric);
        if (positive, negative, zero) != (3, 1, 0) {
            return Err(UpdateError(alloc::format!(
                "metric tensor is not of signature (−, +, +, +): {positive} positive, {negative} \
                 negative and {zero} zero eigenvalues"
            )));
        }
        self.metric = new_metric;
        Ok(())
    }
}
