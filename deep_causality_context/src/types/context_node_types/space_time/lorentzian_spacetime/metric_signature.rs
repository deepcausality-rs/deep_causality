/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{LorentzianSpacetime, MetricSignature};
use deep_causality_algebra::RealField;
use deep_causality_metric::Metric;

impl<R: RealField> MetricSignature for LorentzianSpacetime<R> {
    /// (−,+,+,+), the east-coast convention this type documents and its `interval_squared`
    /// computes. A model in the west-coast convention reports `Metric::Minkowski(4)` from its own
    /// spacetime type; the two are the same signature class under opposite sign conventions.
    fn metric(&self) -> Metric {
        Metric::Lorentzian(4)
    }
}
