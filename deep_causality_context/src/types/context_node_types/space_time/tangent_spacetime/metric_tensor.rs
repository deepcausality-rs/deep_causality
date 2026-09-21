/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TangentSpacetime;
use crate::traits::contextuable::metric_tensor::MetricTensor4D;
use deep_causality_algebra::RealField;

impl<R: RealField> MetricTensor4D for TangentSpacetime<R> {
    fn metric_tensor(&self) -> [[R; 4]; 4] {
        self.metric
    }

    fn update_metric_tensor(&mut self, new_metric: [[R; 4]; 4]) {
        self.metric = new_metric;
    }
}
