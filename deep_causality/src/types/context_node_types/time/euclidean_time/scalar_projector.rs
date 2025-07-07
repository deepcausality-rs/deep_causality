/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EuclideanTime, ScalarProjector, Temporal};

impl ScalarProjector for EuclideanTime {
    type Scalar = f64;

    fn project(&self) -> Self::Scalar {
        self.time_unit()
    }
}
