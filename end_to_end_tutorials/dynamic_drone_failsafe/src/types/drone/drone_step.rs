/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::{Command, Drone, FloatType};
use deep_causality_algebra::Real;

impl Drone {
    /// Flies one second under `command`. Without a satellite fix the drone cannot hold or steer
    /// its ground position and drifts with the wind, whatever it was told.
    pub fn step(&mut self, command: Command) {
        if self.landed {
            return;
        }
        if self.has_gnss_fix() {
            match command {
                Command::Continue => self.y += INSPECTION_SPEED_M_S,
                Command::ReturnHome => self.fly_toward(LINE_ACROSS_M, MISSION_START_ALONG_M),
                Command::Hold | Command::LandNow => {}
            }
        } else {
            self.x += DOWNSLOPE_WIND_M_S;
        }
        if command == Command::LandNow {
            self.descend();
        }
        self.time_s += 1;
    }

    /// Descends for one second, faster once the battery has faulted, and lands on reaching the
    /// ground.
    pub(super) fn descend(&mut self) {
        let rate = if self.battery_faulted() {
            EMERGENCY_DESCENT_M_S
        } else {
            LANDING_DESCENT_M_S
        };
        self.agl_m -= rate;
        if self.agl_m <= ZERO {
            self.agl_m = ZERO;
            self.landed = true;
        }
    }

    fn fly_toward(&mut self, x: FloatType, y: FloatType) {
        let (dx, dy) = (x - self.x, y - self.y);
        let distance = Real::sqrt(dx * dx + dy * dy);
        if distance <= INSPECTION_SPEED_M_S {
            self.x = x;
            self.y = y;
        } else {
            self.x += dx / distance * INSPECTION_SPEED_M_S;
            self.y += dy / distance * INSPECTION_SPEED_M_S;
        }
    }
}
