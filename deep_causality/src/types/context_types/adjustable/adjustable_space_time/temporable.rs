// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::{Temporable, Temporal, TimeScale};

use super::*;

// Type tag required for context.
impl<T> Temporal for AdjustableSpaceTime<T> where T: Copy + Default {}

impl<T> Temporable for AdjustableSpaceTime<T> where T: Copy + Default {
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> u32 {
        self.time_unit
    }
}
