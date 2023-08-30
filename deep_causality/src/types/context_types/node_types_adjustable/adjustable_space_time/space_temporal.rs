// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::SpaceTemporal;

use super::*;

impl<T> SpaceTemporal<T> for AdjustableSpaceTime<T>
where
    T: Default + Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T> + Copy,
{
    fn t(&self) -> &T {
        &self.time_unit
    }
}
