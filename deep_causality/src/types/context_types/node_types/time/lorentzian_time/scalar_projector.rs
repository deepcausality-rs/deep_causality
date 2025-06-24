// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{LorentzianTime, ScalarProjector, Temporal};

impl ScalarProjector for LorentzianTime {
    type Scalar = f64;

    fn project(&self) -> Self::Scalar {
        self.time_unit()
    }
}
