/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{FloatType, Outcome, Surface, Touchdown};

impl Touchdown {
    /// Position across and along the valley, in m.
    pub fn position(&self) -> (FloatType, FloatType) {
        (self.x, self.y)
    }

    pub fn surface(&self) -> Surface {
        self.surface
    }

    /// Slope of the ground at the touchdown point, in degrees.
    pub fn slope_deg(&self) -> FloatType {
        self.slope_deg
    }

    /// Distance to the nearest person, in m.
    pub fn nearest_person_m(&self) -> FloatType {
        self.nearest_person_m
    }

    pub fn outcome(&self) -> Outcome {
        self.outcome
    }

    /// Whether the drone survived on firm, flat ground, clear of people.
    pub fn is_safe(&self) -> bool {
        self.outcome == Outcome::Safe
    }
}
