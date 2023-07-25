// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use crate::prelude::{Adjustable, Identifiable, TimeScale};

pub trait Datable: Adjustable + Identifiable {}

pub trait Temporal: Identifiable + Adjustable {}

// Specializes the `Temporal` trait.
pub trait Temporable: Temporal
{
    fn time_scale(&self) -> TimeScale;
    fn time_unit(&self) -> u32;
}

pub trait Spatial: Identifiable + Adjustable {
    fn x(&self) -> i64;
    fn y(&self) -> i64;
    fn z(&self) -> i64;
}

pub trait SpaceTemporal: Identifiable + Spatial + Temporal + Adjustable {
    fn t(&self) -> u64; // returns 4th dimension, t
}
