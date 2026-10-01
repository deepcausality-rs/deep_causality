/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod drone_getters;
mod drone_guide;
mod drone_scan;
mod drone_step;
mod drone_telemetry;

use crate::constants::*;
use crate::{FloatType, Mission};

/// The inspection drone: the mission it flies, where it is, how high it flies, and whether it has
/// come down, by landing, by falling when its battery died, or by flying into the trees. One step
/// is one second.
#[derive(Debug, Clone)]
pub struct Drone {
    mission: Mission,
    time_s: usize,
    x: FloatType,
    y: FloatType,
    agl_m: FloatType,
    landed: bool,
    fell: bool,
    hit_trees: bool,
}

impl Drone {
    /// At the start of the line, at inspection height, on the night parts 1 to 4 fly.
    pub fn launch() -> Self {
        Self::launch_on(Mission::default())
    }

    /// At the start of the line, at inspection height, on `mission`.
    pub fn launch_on(mission: Mission) -> Self {
        Self {
            mission,
            time_s: 0,
            x: mission.line_across_m(),
            y: MISSION_START_ALONG_M,
            agl_m: INSPECTION_AGL_M,
            landed: false,
            fell: false,
            hit_trees: false,
        }
    }
}
