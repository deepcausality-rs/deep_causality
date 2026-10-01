/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod drone_getters;
mod drone_scan;
mod drone_step;
mod drone_telemetry;

use crate::FloatType;
use crate::constants::*;

/// The inspection drone: where it is, how high it flies, and whether it has landed. One step is
/// one second.
#[derive(Debug, Clone)]
pub struct Drone {
    time_s: usize,
    x: FloatType,
    y: FloatType,
    agl_m: FloatType,
    landed: bool,
}

impl Drone {
    /// At the start of the line, at inspection height.
    pub fn launch() -> Self {
        Self {
            time_s: 0,
            x: LINE_ACROSS_M,
            y: MISSION_START_ALONG_M,
            agl_m: INSPECTION_AGL_M,
            landed: false,
        }
    }
}
