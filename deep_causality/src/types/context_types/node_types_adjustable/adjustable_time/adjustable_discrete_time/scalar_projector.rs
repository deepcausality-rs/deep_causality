// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableDiscreteTime, ScalarProjector, Temporal};

impl ScalarProjector for AdjustableDiscreteTime {
    type Scalar = u64;

    fn project(&self) -> Self::Scalar {
        self.time_unit()
    }
}
