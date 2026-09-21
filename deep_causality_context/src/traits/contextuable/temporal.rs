/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::TimeScale;
use deep_causality_core::Identifiable;

/// Represents entities that have intrinsic temporal properties.
///
/// This trait provides access to both the **scale** (e.g. seconds, minutes)
/// and **unit value** (e.g. timestamp, frame number) associated with a temporal point.
///
/// Use this for any node, edge, or context that evolves over time or contributes
/// to time-dependent reasoning.
///
/// # Notes
/// `TimeUnit` carries no bounds. Implementations range from `()` for a timeless node, through
/// integer ticks, to a real-valued coordinate; ordering and arithmetic are available only where
/// the concrete `TimeUnit` provides them.
pub trait Temporal: Identifiable {
    /// The type this node's time is measured in.
    type TimeUnit;

    /// Returns the unit scale of time (e.g. `TimeScale::Milliseconds`).
    fn time_scale(&self) -> TimeScale;

    /// Returns the time unit value (e.g. 0, 100, 32768).
    fn time_unit(&self) -> Self::TimeUnit;
}
