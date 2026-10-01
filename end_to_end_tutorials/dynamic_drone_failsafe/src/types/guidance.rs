/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::FloatType;

/// What a controller can tell the drone to do relative to the ground below it. The drone flies
/// these by its camera and LiDAR, so it needs no satellite fix for them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Guidance {
    /// Hover over a ground point, in m across and along.
    HoldOver { x: FloatType, y: FloatType },
    /// Fly to a ground point at approach speed, then descend to a height above it, in m, and hover.
    DescendOver {
        x: FloatType,
        y: FloatType,
        agl_m: FloatType,
    },
    /// Fly to a ground point at approach speed, then descend onto it.
    LandOn { x: FloatType, y: FloatType },
}
