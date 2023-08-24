// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::{Constructor, Getters};

use crate::prelude::{Temporable, TimeScale};

mod adjustable;
mod display;
mod identifiable;

// Generic and non-generic time unit field... This is unnecessary.

#[derive(Getters, Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AdjustableTime<T>
where
    T: Copy + Default,
{
    #[getter(name = time_id)] // Rename ID getter to prevent conflict impl with identifiable
    id: u64,
    time_scale: TimeScale,
    time_unit: T,
    time_unit_trait: u32,
}

// test file:
// types/context_types/adjustable/adjustable_time_tests.rs

// Type tag required for context.
impl<T> Temporable for AdjustableTime<T>
where
    T: Copy + Default,
{
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> u32 {
        self.time_unit_trait
    }
}
