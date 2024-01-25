use std::hash::Hash;
// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::ops::*;

use deep_causality_macros::Constructor;

use crate::prelude::TimeScale;

mod display;
mod identifiable;
mod space_temporal;
mod spatial;
mod temporable;

#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct SpaceTime<T>
where
    T: Default
        + Copy
        + Clone
        + Hash
        + Eq
        + PartialEq
        + Add<T, Output = T>
        + Sub<T, Output = T>
        + Mul<T, Output = T>,
{
    id: u64,
    time_scale: TimeScale,
    time_unit: T,
    x: T,
    y: T,
    z: T,
}
