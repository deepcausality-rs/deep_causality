// SPDX-License-Identifier: MIT
// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.
use std::fmt::{Display, Formatter};
use crate::prelude::{Adjustable, Identifiable, SpaceTemporal, Spatial, Temporable, Temporal, TimeScale};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct SpaceTempoid {
    id: u64,
    time_scale: TimeScale,
    time_unit: u32,
    x: i64,
    y: i64,
    z: i64,
}

impl SpaceTempoid {
    pub fn new(id: u64, time_scale: TimeScale, time_unit: u32, x: i64, y: i64, z: i64) -> Self {
        Self { id, time_scale, time_unit, x, y, z }
    }
}

impl Identifiable for SpaceTempoid
{
    fn id(&self) -> u64 {
        self.id
    }
}

// Optional. Override only when needed.
impl Adjustable for SpaceTempoid {}

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
               self.id,
               self.time_scale,
               self.time_unit,
               self.x,
               self.y,
               self.z,
        )
    }
}
