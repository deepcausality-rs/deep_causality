// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableLorentzianSpacetime, Temporal, TimeScale};

impl Temporal<f64> for AdjustableLorentzianSpacetime {
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }
    fn time_unit(&self) -> f64 {
        self.t
    }
}
