/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod terrain_features;
mod terrain_geometry;
mod terrain_thermal;

use crate::FloatType;
use crate::constants::*;

/// The ground under the line: a valley cross-section with a creek on the floor, a 30° grass slope,
/// a flat terrace and a stand of trees on it, a flat access road, a 25° rock slope above it, flat
/// pads under the towers, a ravine, and the people on the ground. `x` runs across the slope from the
/// creek uphill, `y` along the valley; both in metres.
#[derive(Debug, Clone, PartialEq)]
pub struct Terrain {
    line_across_m: FloatType,
    terrace_across_m: (FloatType, FloatType),
    terrace_along_m: (FloatType, FloatType),
    woodland_across_m: (FloatType, FloatType),
    woodland_along_m: (FloatType, FloatType),
    crew_m: Vec<(FloatType, FloatType)>,
}

impl Terrain {
    /// The ground the default night flies over: the line 60 m across, the terrace 24-36 m across
    /// and 420-470 m along, trees 64-104 m across and 520-600 m along, four of the crew on the road
    /// and two at the base of the second tower.
    pub fn new() -> Self {
        Self::with(
            LINE_ACROSS_M,
            (TERRACE_ACROSS_M, TERRACE_ALONG_M),
            (WOODLAND_ACROSS_M, WOODLAND_ALONG_M),
            CREW_M.to_vec(),
        )
    }

    /// Ground with the line, and so the tower pads, at `line_across_m`, the terrace and the trees
    /// each between the given distances across and along, and a person at each point of `crew_m`;
    /// all in m.
    pub fn with(
        line_across_m: FloatType,
        terrace_m: ((FloatType, FloatType), (FloatType, FloatType)),
        woodland_m: ((FloatType, FloatType), (FloatType, FloatType)),
        crew_m: Vec<(FloatType, FloatType)>,
    ) -> Self {
        Self {
            line_across_m,
            terrace_across_m: terrace_m.0,
            terrace_along_m: terrace_m.1,
            woodland_across_m: woodland_m.0,
            woodland_along_m: woodland_m.1,
            crew_m,
        }
    }
}

impl Default for Terrain {
    fn default() -> Self {
        Self::new()
    }
}
