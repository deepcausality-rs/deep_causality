/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{FloatType, Surface};

/// What became of the drone after touchdown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    /// It landed upright on ground it can stay on.
    Safe,
    /// It touched down on water.
    Ditched,
    /// It came down into the ravine.
    IntoRavine,
    /// It came down too close to a person.
    AmongPeople,
    /// The ground was too steep: it tipped over and tumbled downhill until the ground flattened.
    TippedAndRolled {
        distance_m: FloatType,
        came_to_rest_on: Surface,
    },
}
