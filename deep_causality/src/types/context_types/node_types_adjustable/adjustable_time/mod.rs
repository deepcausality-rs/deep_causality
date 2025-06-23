// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.

use deep_causality_macros::Constructor;
use std::fmt::Debug;

use crate::prelude::TimeScale;

mod adjustable;
mod display;
mod getters;
mod identifiable;
mod temporable;

#[derive(Constructor, Debug, Copy, Clone, Eq, PartialEq)]
pub struct AdjustableTime {
    id: u64,
    time_scale: TimeScale,
    time_unit: u64,
}
