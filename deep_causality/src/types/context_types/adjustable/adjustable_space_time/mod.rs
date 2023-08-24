// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use std::marker::PhantomData;

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
    T: Copy + Default,
{
    #[getter(name = time_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    time_scale: TimeScale,
    time_unit: u32,
    x: i64,
    y: i64,
    z: i64,
    ty: PhantomData<T>, // Need to bind T
}
