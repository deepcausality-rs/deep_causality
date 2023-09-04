// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::ops::{Add, Mul, Sub};

use crate::prelude::Spatial;

use super::*;

impl<T> Spatial<T> for Space<T>
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
