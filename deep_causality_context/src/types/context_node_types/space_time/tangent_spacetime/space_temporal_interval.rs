/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::traits::contextuable::metric_tensor::MetricTensor4D;
use crate::{SpaceTemporalInterval, TangentSpacetime};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

impl<R: RealField + FromPrimitive> SpaceTemporalInterval for TangentSpacetime<R> {
    fn time(&self) -> R {
        // `t` is in seconds by construction.
        self.t
    }
    fn position(&self) -> [R; 3] {
        [self.x, self.y, self.z]
    }
    /// `g_ab Δxᵃ Δxᵇ` with `self`'s tensor, on the displacement `(Δt, Δx, Δy, Δz)` in the
    /// coordinate order the tensor is indexed in. See [`TangentSpacetime`] for how this relates to
    /// the interval.
    fn interval_squared(&self, other: &Self) -> R {
        let dt = self.t - other.t;
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;

        let v = [dt, dx, dy, dz];
        let g = self.metric_tensor();

        let mut sum = R::zero();
        for u in 0..4 {
            for w in 0..4 {
                sum += g[u][w] * v[u] * v[w];
            }
        }
        sum
    }
}
