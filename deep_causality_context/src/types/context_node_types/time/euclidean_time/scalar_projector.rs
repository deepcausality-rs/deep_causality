/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EuclideanTime, ScalarProjector, Temporal};
use deep_causality_algebra::RealField;

impl<R: RealField + Default> ScalarProjector for EuclideanTime<R> {
    type Scalar = R;

    fn project(&self) -> Self::Scalar {
        self.time_unit()
    }
}
