// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::fmt::Debug;
use std::hash::Hash;
use std::ops::*;

use deep_causality_macros::Constructor;

use crate::prelude::TimeScale;

mod adjustable;
mod display;
mod getters;
mod identifiable;
mod temporable;

#[derive(Constructor, Debug, Copy, Clone, Eq, PartialEq)]
pub struct AdjustableTime<T>
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
    id: u64,
    time_scale: TimeScale,
    time_unit: T,
}
