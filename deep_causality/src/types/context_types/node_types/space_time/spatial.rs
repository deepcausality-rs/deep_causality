// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::ops::{Add, Mul, Sub};

use crate::prelude::Spatial;
use crate::types::context_types::node_types::space_time::SpaceTempoid;

impl<T> Spatial<T> for SpaceTempoid<T>
where
    T: Default + Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T>,
{
    fn x(&self) -> &T {
        &self.x
    }

    fn y(&self) -> &T {
        &self.y
    }

    fn z(&self) -> &T {
        &self.z
    }
}
