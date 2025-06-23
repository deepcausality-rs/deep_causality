// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableTangentSpacetime, MetricTensor4D};

impl MetricTensor4D for AdjustableTangentSpacetime {
    fn metric_tensor(&self) -> [[f64; 4]; 4] {
        self.metric
    }

    fn update_metric_tensor(&mut self, new_metric: [[f64; 4]; 4]) {
        self.metric = new_metric;
    }
}