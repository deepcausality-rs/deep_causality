// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
//

use std::fmt::Debug;
use std::hash::Hash;
use deep_causality_macros::Constructor;

use crate::prelude::TimeScale;

mod display;
mod identifiable;
mod temporable;

#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct Time<T>
where
    T: Copy + Clone + Hash + Eq + PartialEq +Debug,
{
    id: u64,
    time_scale: TimeScale,
    time_unit: T,
}
