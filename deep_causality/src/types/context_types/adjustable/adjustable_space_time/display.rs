// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Display, Formatter};

use super::*;

impl<T> Display for AdjustableSpaceTime<T>
where
    T: Copy + Default + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AdjustableSpaceTime<{}> {{ id: {}, time_scale: {}, time_unit: {}, x: {}, y: {}, z: {} }}",
               T::default(), self.id, self.time_scale, self.time_unit, self.x, self.y, self.z)
    }
}
