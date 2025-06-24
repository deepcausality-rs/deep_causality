// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{EuclideanTime, ScalarProjector, Temporal};

impl ScalarProjector for EuclideanTime {
    type Scalar = f64;

    fn project(&self) -> Self::Scalar {
        self.time_unit()
    }
}