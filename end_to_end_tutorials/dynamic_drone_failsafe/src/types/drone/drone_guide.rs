/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::{Drone, FloatType, Guidance, Terrain};
use deep_causality_algebra::Real;

impl Drone {
    /// Flies one second under `guidance` over `terrain`, holding or steering its ground position
    /// against the wind by its camera and LiDAR. On a landing it descends only once it is over the
    /// point, and faster once the battery has faulted.
    pub fn guide(&mut self, guidance: Guidance, terrain: &Terrain) {
        if self.landed {
            return;
        }
        match guidance {
            Guidance::HoldOver { x, y } => {
                self.approach(x, y);
            }
            Guidance::DescendOver { x, y, agl_m } => {
                if self.approach(x, y) && self.agl_m > agl_m {
                    let lower = self.agl_m - LANDING_DESCENT_M_S;
                    self.agl_m = if lower > agl_m { lower } else { agl_m };
                }
            }
            Guidance::LandOn { x, y } => {
                if self.approach(x, y) {
                    self.descend();
                }
            }
        }
        self.tick(terrain);
    }

    /// Moves toward a ground point at approach speed, and reports whether the drone is over it.
    fn approach(&mut self, x: FloatType, y: FloatType) -> bool {
        let (dx, dy) = (x - self.x, y - self.y);
        let distance = Real::sqrt(dx * dx + dy * dy);
        if distance <= APPROACH_SPEED_M_S {
            self.x = x;
            self.y = y;
            true
        } else {
            self.x += dx / distance * APPROACH_SPEED_M_S;
            self.y += dy / distance * APPROACH_SPEED_M_S;
            false
        }
    }
}
