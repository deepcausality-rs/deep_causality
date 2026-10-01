/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod terrain_features;
mod terrain_geometry;
mod terrain_thermal;

/// The ground under the line: a valley cross-section with a creek on the floor, a 30° grass slope,
/// a flat terrace above the creek, a flat access road, a 25° rock slope above it, flat pads under
/// the towers, a ravine, and people: a crew on the road and two workers at a tower. `x` runs across the slope from the creek uphill, `y` along the
/// valley; both in metres.
#[derive(Debug, Clone, Copy, Default)]
pub struct Terrain;

impl Terrain {
    pub fn new() -> Self {
        Self
    }
}
