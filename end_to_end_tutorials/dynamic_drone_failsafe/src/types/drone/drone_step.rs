/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::{Command, Drone, FloatType, Terrain};
use deep_causality_algebra::Real;

impl Drone {
    /// Flies one second under `command` over `terrain`. Without a satellite fix the drone cannot
    /// hold or steer its ground position and drifts with the wind, whatever it was told.
    pub fn step(&mut self, command: Command, terrain: &Terrain) {
        if self.landed {
            return;
        }
        if self.has_gnss_fix() {
            match command {
                Command::Continue => {
                    self.y += INSPECTION_SPEED_M_S;
                    let higher = self.agl_m + CLIMB_M_S;
                    self.agl_m = if higher < INSPECTION_AGL_M {
                        higher
                    } else {
                        INSPECTION_AGL_M
                    };
                }
                Command::ReturnHome => {
                    self.fly_toward(self.mission.line_across_m(), MISSION_START_ALONG_M)
                }
                Command::Hold | Command::LandNow => {}
            }
        } else {
            let (wx, wy) = self.mission.wind_m_s();
            self.x += wx;
            self.y += wy;
        }
        if command == Command::LandNow {
            self.descend();
        }
        self.tick(terrain);
    }

    /// Ends the second. A drone below the canopy over trees has flown into them; a drone still in
    /// the air when its battery dies falls where it is.
    pub(super) fn tick(&mut self, terrain: &Terrain) {
        self.time_s += 1;
        if self.landed {
            return;
        }
        if self.agl_m < terrain.canopy_m(self.x, self.y) {
            self.landed = true;
            self.hit_trees = true;
        } else if self.battery_dead() {
            self.landed = true;
            self.fell = true;
        }
    }

    /// Descends for one second, faster once the battery has faulted. On reaching the ground the
    /// mission's gust moves the drone before it lands.
    pub(super) fn descend(&mut self) {
        let rate = if self.battery_faulted() {
            EMERGENCY_DESCENT_M_S
        } else {
            LANDING_DESCENT_M_S
        };
        self.agl_m -= rate;
        if self.agl_m <= ZERO {
            let (gx, gy) = self.mission.touchdown_gust_m();
            self.x += gx;
            self.y += gy;
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
