/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{MetricSignature, TangentSpacetime};
use deep_causality_algebra::RealField;
use deep_causality_metric::Metric;

impl<R: RealField> MetricSignature for TangentSpacetime<R> {
    /// (−,+,+,+). The stored tensor starts at `diag(-c², 1, 1, 1)` and may be replaced through
    /// [`MetricTensor4D::update_metric_tensor`](crate::MetricTensor4D::update_metric_tensor), but
    /// a signature does not vary under continuous evolution. Every component of the tensor can
    /// change while this answer stays the same, which is why it is derived from the type rather
    /// than read off the field.
    fn metric(&self) -> Metric {
        Metric::Lorentzian(4)
    }
}
