// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::{AdjustableTangentSpacetime, Temporal, TimeScale};

impl Temporal<f64> for AdjustableTangentSpacetime {
    fn time_scale(&self) -> TimeScale {
        TimeScale::Second
    }
    fn time_unit(&self) -> &f64 {
        &self.t
    }
}
