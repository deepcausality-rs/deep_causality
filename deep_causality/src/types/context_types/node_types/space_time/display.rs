// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Mul, Sub};

use crate::types::context_types::node_types::space_time::SpaceTempoid;

impl<T> Display for SpaceTempoid<T>
where
    T: Copy + Debug + Default + Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SpaceTempoid: id={}, time_scale={:?}, time_unit={:?}, x={:?}, y={:?}, z={:?}",
            self.id, self.time_scale, self.time_unit, self.x, self.y, self.z,
        )
    }
}
