/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::{FloatType, Surface, Terrain};

impl Terrain {
    /// Temperature of the bare ground at a point at local `hour`, in °C. Land cools faster than
    /// water at night, so the creek is the warmest ground by night and the coolest by day.
    pub fn ground_temperature_c(&self, x: FloatType, y: FloatType, hour: FloatType) -> FloatType {
        let night = is_night(hour);
        match (self.surface(x, y), night) {
            (Surface::Water, true) => NIGHT_WATER_C,
            (Surface::Water, false) => DAY_WATER_C,
            (Surface::Grass, true) => NIGHT_GRASS_C,
            (Surface::Grass, false) => DAY_GRASS_C,
            (Surface::Road, true) => NIGHT_ROAD_C,
            (Surface::Road, false) => DAY_ROAD_C,
            (Surface::Pad, true) => NIGHT_PAD_C,
            (Surface::Pad, false) => DAY_PAD_C,
            (Surface::Rock | Surface::Ravine, true) => NIGHT_ROCK_C,
            (Surface::Rock | Surface::Ravine, false) => DAY_ROCK_C,
        }
    }
}

/// Whether local `hour` falls between dusk and dawn.
fn is_night(hour: FloatType) -> bool {
    let hour = hour - deep_causality_algebra::Real::floor(hour / HOURS_PER_DAY) * HOURS_PER_DAY;
    hour >= DUSK_HOUR || hour < DAWN_HOUR
}
