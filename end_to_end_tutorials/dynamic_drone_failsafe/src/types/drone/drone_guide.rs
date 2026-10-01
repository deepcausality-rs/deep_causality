/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::constants::*;
use crate::{Drone, FloatType, Guidance};
use deep_causality_algebra::Real;

impl Drone {
    /// Flies one second under `guidance`, holding or steering its ground position against the
    /// wind by its camera and LiDAR. On a landing it descends only once it is over the point, and
    /// faster once the battery has faulted.
    pub fn guide(&mut self, guidance: Guidance) {
        if self.landed {
            return;
        }
        match guidance {
            Guidance::HoldOver { x, y } => {
                self.approach(x, y);
            }
            Guidance::LandOn { x, y } => {
                if self.approach(x, y) {
                    self.descend();
                }
            }
        }
        self.time_s += 1;
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
