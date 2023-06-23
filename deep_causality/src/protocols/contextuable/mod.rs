/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use crate::prelude::{Adjustable, Identifiable};

pub trait Temporal: Identifiable + Adjustable {}

pub trait Spatial: Identifiable + Adjustable {
    fn x(&self) -> i64;
    fn y(&self) -> i64;
    fn z(&self) -> i64;
}

pub trait SpaceTemporal: Identifiable + Spatial + Temporal + Adjustable {
    fn t(&self) -> i64; // returns 4th dimension, t
}

pub trait Datable: Adjustable + Identifiable {}
