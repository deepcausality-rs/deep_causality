/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::{Drone, FloatType, Telemetry};
use deep_causality_num::lift_usize;

impl Drone {
    /// Whether satellite positioning still holds a fix.
    pub fn has_gnss_fix(&self) -> bool {
        self.time_s < GNSS_LOST_AT_S
    }

    /// Whether a cell has failed.
    pub(super) fn battery_faulted(&self) -> bool {
        self.time_s >= BATTERY_FAULT_AT_S
    }

    /// The telemetry for the current second, from the fault timeline.
    pub fn telemetry(&self) -> Telemetry {
        let t = self.time_s;
        let (satellites, hdop) = if t >= GNSS_LOST_AT_S {
            (SATELLITES_LOST, HDOP_LOST)
        } else if t >= GNSS_DEGRADED_AT_S {
            (SATELLITES_DEGRADED, HDOP_DEGRADED)
        } else {
            (SATELLITES_NOMINAL, HDOP_NOMINAL)
        };
        let link_loss_pct = if t >= LINK_LOST_AT_S {
            LINK_LOSS_LOST_PCT
        } else {
            LINK_LOSS_NOMINAL_PCT
        };
        let min_cell_v = if self.battery_faulted() {
            CELL_V_FAULT
        } else {
            CELL_V_LAUNCH - CELL_V_DROP_PER_S * lift_usize::<FloatType>(t)
        };
        Telemetry::new(t, satellites, hdop, link_loss_pct, min_cell_v, self.agl_m)
    }
}
