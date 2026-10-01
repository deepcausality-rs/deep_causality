/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod telemetry_getters;

use crate::FloatType;

/// What the drone reports each second.
#[derive(Debug, Clone, Copy, Default)]
pub struct Telemetry {
    time_s: usize,
    satellites: FloatType,
    hdop: FloatType,
    link_loss_pct: FloatType,
    min_cell_v: FloatType,
    altitude_agl_m: FloatType,
}

impl Telemetry {
    pub fn new(
        time_s: usize,
        satellites: FloatType,
        hdop: FloatType,
        link_loss_pct: FloatType,
        min_cell_v: FloatType,
        altitude_agl_m: FloatType,
    ) -> Self {
        Self {
            time_s,
            satellites,
            hdop,
            link_loss_pct,
            min_cell_v,
            altitude_agl_m,
        }
    }
}
