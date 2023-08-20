// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . The DeepCausality Authors. All Rights Reserved.
use std::fmt::{Display, Formatter};

use deep_causality_macros::Constructor;

use crate::prelude::{Identifiable, SpaceTemporal, Spatial, Temporable, Temporal, TimeScale};

#[derive(Constructor, Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct SpaceTempoid {
    id: u64,
    time_scale: TimeScale,
    time_unit: u32,
    x: i64,
    y: i64,
    z: i64,
}

impl Identifiable for SpaceTempoid
{
    fn id(&self) -> u64 {
        self.id
    }
}

impl Temporal for SpaceTempoid {}


impl Temporable for SpaceTempoid
{
    fn time_scale(&self) -> TimeScale {
        self.time_scale
    }

    fn time_unit(&self) -> u32 {
        self.time_unit
    }
}


impl Spatial for SpaceTempoid
{
    fn x(&self) -> i64 {
        self.x
    }

    fn y(&self) -> i64 {
        self.y
    }

    fn z(&self) -> i64 {
        self.z
    }
}


impl SpaceTemporal for SpaceTempoid
{
    fn t(&self) -> u64 {
        self.time_unit as u64
    }
}


impl Display for SpaceTempoid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpaceTempoid: id={}, time_scale={}, time_unit={}, x={}, y={}, z={}",
               self.id, self.time_scale, self.time_unit, self.x, self.y, self.z,
        )
    }
}
