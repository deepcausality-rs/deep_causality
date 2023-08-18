// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use crate::prelude::{Temporal, TimeScale};

mod identifiable;
mod display;
mod adjustable;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AdjustableTime<T>
    where T: Copy + Default,
{
    id: u64,
    time_scale: TimeScale,
    time_unit: T,
}

impl<T> AdjustableTime<T>
    where T: Copy + Default,
{
    pub fn new(id: u64, time_scale: TimeScale, time_unit: T) -> Self {
        Self { id, time_scale, time_unit }
    }
}

impl<T> AdjustableTime<T>
    where T: Copy + Default,
{
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn time_scale(&self) -> TimeScale {
        self.time_scale
    }
    pub fn time_unit(&self) -> T {
        self.time_unit
    }
}

// Type tag required for context.
impl<T> Temporal for AdjustableTime<T>
    where T: Copy + Default {}
