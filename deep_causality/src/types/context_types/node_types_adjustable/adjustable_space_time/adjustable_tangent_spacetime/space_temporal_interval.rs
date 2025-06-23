// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//
use crate::prelude::{AdjustableTangentSpacetime, MetricTensor4D, SpaceTemporalInterval};

impl SpaceTemporalInterval for AdjustableTangentSpacetime {
    fn time(&self) -> f64 {
        self.t
    }
    fn position(&self) -> [f64; 3] {
        [self.x, self.y, self.z]
    }
    // Override `interval_squared()` for curved spacetime
    fn interval_squared(&self, other: &Self) -> f64 {
        let dt = self.t - other.t;
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;

        let v = [dt, dx, dy, dz];
        let g = self.metric_tensor();

        let mut sum = 0.0;
        for u in 0..4 {
            for w in 0..4 {
                sum += g[u][w] * v[u] * v[w];
            }
        }
        sum
    }
}
