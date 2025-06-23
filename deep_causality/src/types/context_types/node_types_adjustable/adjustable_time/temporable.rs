// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::Debug;
use std::hash::Hash;
use std::ops::{Add, Mul, Sub};
use crate::prelude::{AdjustableTime, TimeScale};
use crate::traits::contextuable::temporal::Temporal;

impl<T> Temporal<T> for AdjustableTime<T>
where
    T: Debug
        + Default
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Mul<T, Output = T>
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + PartialOrd,
{
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> &T {
        &self.time_unit
    }
}
