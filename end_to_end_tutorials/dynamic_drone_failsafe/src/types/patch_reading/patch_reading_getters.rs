/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{FloatType, PatchReading, Quantity};

impl PatchReading {
    /// The patch's grid index: across and along the valley, counted in patch sides.
    pub fn patch(&self) -> (i64, i64) {
        self.patch
    }

    /// The patch's centre, across and along the valley, in m.
    pub fn centre(&self) -> (FloatType, FloatType) {
        self.centre
    }

    /// The measured value of `quantity`.
    pub fn value(&self, quantity: Quantity) -> FloatType {
        self.values[quantity as usize]
    }

    /// One standard deviation of the measured value of `quantity`.
    pub fn sigma(&self, quantity: Quantity) -> FloatType {
        self.sigmas[quantity as usize]
    }
}
