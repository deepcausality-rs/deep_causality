/*
 * Copyright (c) 2023. Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
 */

use crate::prelude::{Adjustable, Identifiable, Spatial, Temporal};

pub trait SpaceTemporal: Identifiable + Spatial + Temporal + Adjustable {
    fn t(&self) -> i64; // returns 4th dimension, t
}
