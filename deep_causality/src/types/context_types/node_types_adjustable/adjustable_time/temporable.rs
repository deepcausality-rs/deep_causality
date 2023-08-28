// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::ops::{Add, Mul, Sub};

use crate::prelude::{AdjustableTime, Temporable, TimeScale};

impl<T> Temporable<T> for AdjustableTime<T>
where
    T: Default + Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T> + Copy,
{
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> &T {
        &self.time_unit
    }
}
