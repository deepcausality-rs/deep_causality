// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::prelude::{Temporal, TimeScale};

// Specializes the `Temporal` trait.
pub trait Temporable: Temporal
{
    fn time_scale(&self) -> TimeScale;
    fn time_unit(&self) -> u32;
}