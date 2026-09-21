/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{MetricSignature, NoSpaceTime};
use deep_causality_algebra::RealField;
use deep_causality_metric::Metric;

impl<R: RealField> MetricSignature for NoSpaceTime<R> {
    /// Zero dimensions, so there are no axes to give a sign to.
    fn metric(&self) -> Metric {
        Metric::Euclidean(0)
    }
}
