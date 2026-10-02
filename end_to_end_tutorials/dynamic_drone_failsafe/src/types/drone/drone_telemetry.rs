/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::{Drone, FloatType, Telemetry};
use deep_causality_num::{lift_usize, lower};

impl Drone {
    /// Whether satellite positioning still holds a fix.
    pub fn has_gnss_fix(&self) -> bool {
        !self.mission.faults().gnss_lost(self.time_s)
    }

    /// Whether a cell has failed.
    pub(super) fn battery_faulted(&self) -> bool {
        self.mission.faults().battery_faulted(self.time_s)
    }

    /// Whether the battery has died: this long after a cell fails, or once the lowest cell has run
    /// down to empty.
    pub(super) fn battery_dead(&self) -> bool {
        let after_fault = self
            .mission
            .faults()
            .battery_fault_at_s()
            .is_some_and(|at| self.time_s >= at + BATTERY_LIFE_AFTER_FAULT_S);
        after_fault || self.healthy_cell_v() < CELL_V_EMPTY
    }

    /// The lowest cell's voltage from use alone, without a failed cell.
    fn healthy_cell_v(&self) -> FloatType {
        CELL_V_LAUNCH - CELL_V_DROP_PER_S * lift_usize::<FloatType>(self.time_s)
    }

    /// The telemetry for the current second, from the fault timeline.
    pub fn telemetry(&self) -> Telemetry {
        let t = self.time_s;
        let faults = self.mission.faults();
        let (satellites, hdop) = if faults.gnss_lost(t) {
            (SATELLITES_LOST, HDOP_LOST)
        } else if faults.gnss_degraded(t) {
            (SATELLITES_DEGRADED, HDOP_DEGRADED)
        } else {
            (SATELLITES_NOMINAL, HDOP_NOMINAL)
        };
        let link_loss_pct = if faults.link_lost(t) {
            LINK_LOSS_LOST_PCT
        } else {
            LINK_LOSS_NOMINAL_PCT
        };
        let min_cell_v = if self.battery_faulted() {
            CELL_V_FAULT
        } else {
            self.healthy_cell_v()
        };
        let clock_s = lower(self.mission.start_hour() * SECONDS_PER_HOUR) as u64 + t as u64;
        Telemetry::new(
            t,
            clock_s,
            satellites,
            hdop,
            link_loss_pct,
            min_cell_v,
            self.agl_m,
        )
    }
}
