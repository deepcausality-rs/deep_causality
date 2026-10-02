/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

/// What a controller can tell the drone to do.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Command {
    /// Fly the inspection mission.
    #[default]
    Continue,
    /// Stop and hover.
    Hold,
    /// Fly back to the launch point.
    ReturnHome,
    /// Descend and land where the drone is.
    LandNow,
}
