/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EuclideanSpacetime, MetricSignature};
use deep_causality_algebra::RealField;
use deep_causality_metric::Metric;

impl<R: RealField> MetricSignature for EuclideanSpacetime<R> {
    /// (+,+,+,+). This is the Newtonian spacetime: flat orthogonal space and an absolute clock,
    /// so the time axis squares positive like the spatial ones and there is no light cone.
    fn metric(&self) -> Metric {
        Metric::Euclidean(4)
    }
}
