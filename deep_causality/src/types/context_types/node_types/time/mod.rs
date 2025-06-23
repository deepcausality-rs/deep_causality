// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use deep_causality_macros::Constructor;
use std::fmt::Debug;
use std::hash::Hash;

use crate::prelude::TimeScale;

mod display;
mod identifiable;
mod temporable;

#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Time {
    id: u64,
    time_scale: TimeScale,
    time_unit: u64,
}
