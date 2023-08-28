// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Debug, Display};
use std::ops::{Add, Mul, Sub};

use crate::types::context_types::node_types::time::Tempoid;

impl<T> Display for Tempoid<T>
where
    T: Debug + Default + Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tempoid: id: {}, time_scale: {}, time_unit: {:?}",
            self.id, self.time_scale, self.time_unit
        )
    }
}
