/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// What the downward sensors measure on each ground patch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantity {
    /// The patch's mean temperature, from the thermal camera, in °C.
    MeanTemperature,
    /// The patch's hottest point, from the thermal camera, in °C.
    HotSpot,
    /// The ground's slope across the patch, from LiDAR, in degrees.
    Slope,
    /// The share of LiDAR pulses that return from the patch.
    Returns,
    /// The tallest object standing on the patch, from LiDAR, in m.
    Protrusion,
}

impl Quantity {
    pub const ALL: [Quantity; 5] = [
        Quantity::MeanTemperature,
        Quantity::HotSpot,
        Quantity::Slope,
        Quantity::Returns,
        Quantity::Protrusion,
    ];
}
