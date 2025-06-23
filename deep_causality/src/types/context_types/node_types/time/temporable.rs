// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use crate::prelude::TimeScale;
use crate::traits::contextuable::temporal::Temporal;
use crate::types::context_types::node_types::time::Time;

impl Temporal<u64> for Time {
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> &u64 {
        &self.time_unit
    }
}
