/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{MetricSignature, MinkowskiSpacetime};
use deep_causality_algebra::RealField;
use deep_causality_metric::Metric;

impl<R: RealField> MetricSignature for MinkowskiSpacetime<R> {
    /// `Metric::Lorentzian(4)`: `η = diag(−1, +1, +1, +1)` with generator 0 the time axis, the
    /// coordinate order of this type (Carroll 1997, eq. (1.8)).
    fn metric(&self) -> Metric {
        Metric::Lorentzian(4)
    }
}
