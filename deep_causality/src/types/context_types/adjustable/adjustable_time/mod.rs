use deep_causality_macros::{Constructor, Getters};

// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use crate::prelude::{Temporal, TimeScale};

mod adjustable;
mod display;
mod identifiable;

#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AdjustableTime<T>
    where T: Copy + Default,
{
    #[getter(name = time_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    time_scale: TimeScale,
    time_unit: T,
}

// Type tag required for context.
impl<T> Temporal for AdjustableTime<T> where T: Copy + Default {}
