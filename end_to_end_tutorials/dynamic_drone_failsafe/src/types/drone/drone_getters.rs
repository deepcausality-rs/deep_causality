/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Drone, FloatType, Mission};

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

    /// Whether the drone has come down, by landing or by falling.
    pub fn landed(&self) -> bool {
        self.landed
    }

    /// Whether the drone fell because its battery died in the air.
    pub fn fell(&self) -> bool {
        self.fell
    }

    /// Whether the drone flew or descended into the trees.
    pub fn hit_trees(&self) -> bool {
        self.hit_trees
    }

    /// The mission the drone flies.
    pub fn mission(&self) -> Mission {
        self.mission
    }
}
