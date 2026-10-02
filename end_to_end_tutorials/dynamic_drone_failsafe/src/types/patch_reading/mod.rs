/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

mod patch_reading_getters;

use crate::FloatType;

/// One frame's reading of one ground patch: each quantity's value and one standard deviation of
/// it, in the order of [`Quantity::ALL`](crate::Quantity::ALL).
#[derive(Debug, Clone, Copy)]
pub struct PatchReading {
    patch: (i64, i64),
    centre: (FloatType, FloatType),
    values: [FloatType; 5],
    sigmas: [FloatType; 5],
}

impl PatchReading {
    pub fn new(
        patch: (i64, i64),
        centre: (FloatType, FloatType),
        values: [FloatType; 5],
        sigmas: [FloatType; 5],
    ) -> Self {
        Self {
            patch,
            centre,
            values,
            sigmas,
        }
    }
}
