/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{MetricSignature, NewtonianSpacetime};
use deep_causality_algebra::RealField;
use deep_causality_metric::Metric;

impl<R: RealField> MetricSignature for NewtonianSpacetime<R> {
    /// The signature (0, 1, 1, 1) of the spatial metric `ĥ_ab` relative to absolute space
    /// (Malament 2012, §4.1, Proposition 4.1.2), with generator 0 the time axis, the coordinate
    /// order of this type. The metric crate names this sign pattern `Metric::PGA(4)`: one
    /// degenerate generator first, three positive after it. The temporal metric `t_ab`, of
    /// signature (1, 0, 0, 0), is not reported.
    fn metric(&self) -> Metric {
        Metric::PGA(4)
    }
}
