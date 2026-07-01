/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::satelite_types::SatId;
use chrono::NaiveDateTime;
use deep_causality_num::RealField;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct OrbitData<R>
where
    R: RealField,
{
    timestamp: NaiveDateTime,
    sat_id: SatId,
    x_m: R, // Meters
    y_m: R, // Meters
    z_m: R, // Meters
}

impl<R> OrbitData<R>
where
    R: RealField,
{
    pub fn new(timestamp: NaiveDateTime, sat_id: SatId, x_m: R, y_m: R, z_m: R) -> Self {
        Self {
            timestamp,
            sat_id,
            x_m,
            y_m,
            z_m,
        }
    }
}

impl<R> OrbitData<R>
where
    R: RealField + Clone,
{
    pub fn timestamp(&self) -> NaiveDateTime {
        self.timestamp
    }

    pub fn sat_id(&self) -> &SatId {
        &self.sat_id
    }

    pub fn x_m(&self) -> R {
        self.x_m
    }

    pub fn y_m(&self) -> R {
        self.y_m
    }

    pub fn z_m(&self) -> R {
        self.z_m
    }

    /// Compute radius from Earth center
    pub fn radius_m(&self) -> R {
        let x2 = self.x_m * self.x_m;
        let y2 = self.y_m * self.y_m;
        let z2 = self.z_m * self.z_m;
        (x2 + y2 + z2).sqrt()
    }
}

impl<R> Display for OrbitData<R>
where
    R: RealField + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OrbitData: timestamp: {}, Satellite ID: {}, X: {}, Y: {}, Z: {}",
            self.timestamp, self.sat_id, self.x_m, self.y_m, self.z_m
        )
    }
}
