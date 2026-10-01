/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{FloatType, Telemetry};

impl Telemetry {
    /// Seconds since launch.
    pub fn time_s(&self) -> usize {
        self.time_s
    }

    /// Satellites the receiver tracks.
    pub fn satellites(&self) -> FloatType {
        self.satellites
    }

    /// Horizontal dilution of precision of the satellite fix.
    pub fn hdop(&self) -> FloatType {
        self.hdop
    }

    /// Command-link packet loss, in percent.
    pub fn link_loss_pct(&self) -> FloatType {
        self.link_loss_pct
    }

    /// Voltage of the weakest cell in the pack.
    pub fn min_cell_v(&self) -> FloatType {
        self.min_cell_v
    }

    /// Height above the ground, in m.
    pub fn altitude_agl_m(&self) -> FloatType {
        self.altitude_agl_m
    }
}
