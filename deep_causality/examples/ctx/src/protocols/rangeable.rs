// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.


use deep_causality::prelude::Datable;

use crate::types::bar_range::BarRange;

pub trait Rangeable: Datable
{
    fn data_range(&self) -> BarRange;
}
