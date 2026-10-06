/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{MinkowskiTime, ScalarProjector, Temporal};
use deep_causality_algebra::RealField;

impl<R: RealField + Default> ScalarProjector for MinkowskiTime<R> {
    type Scalar = R;

    fn project(&self) -> Self::Scalar {
        self.time_unit()
    }
}
