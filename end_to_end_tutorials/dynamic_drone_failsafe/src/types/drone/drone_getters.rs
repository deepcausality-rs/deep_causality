/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Drone, FloatType};

impl Drone {
    /// Seconds since launch.
    pub fn time_s(&self) -> usize {
        self.time_s
    }

    /// Position across and along the valley, in m.
    pub fn position(&self) -> (FloatType, FloatType) {
        (self.x, self.y)
    }

    /// Height above the ground, in m.
    pub fn altitude_agl_m(&self) -> FloatType {
        self.agl_m
    }

    pub fn landed(&self) -> bool {
        self.landed
    }
}
