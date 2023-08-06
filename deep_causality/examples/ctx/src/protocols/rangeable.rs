// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.


use deep_causality::prelude::Datable;
use crate::types::bar_range::BarRange;

pub trait Rangeable: Datable
{
    fn data_range(&self) -> BarRange;
}
