/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{MetricSignature, TangentSpacetime};
use deep_causality_algebra::RealField;
use deep_causality_metric::Metric;

impl<R: RealField> MetricSignature for TangentSpacetime<R> {
    /// `Metric::Lorentzian(4)`, (−, +, +, +) with generator 0 the time axis. Every tensor this
    /// node holds has that signature: the default `diag(−c², 1, 1, 1)`, and any tensor
    /// [`update_metric_tensor`](crate::MetricTensor4D::update_metric_tensor) accepts.
    fn metric(&self) -> Metric {
        Metric::Lorentzian(4)
    }
}
