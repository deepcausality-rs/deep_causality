// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::ops::*;

use deep_causality_macros::{Constructor, Getters};

use crate::prelude::TimeScale;

mod adjustable;
mod display;
mod identifiable;
mod space_temporal;
mod spatial;
mod temporable;

#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AdjustableSpaceTime<T>
where
    T: Default + Add<T, Output = T> + Sub<T, Output = T> + Mul<T, Output = T> + Copy,
{
    #[getter(name = time_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    time_scale: TimeScale,
    time_unit: T,
    x: T,
    y: T,
    z: T,
}
