/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::GalileanSpacetime;
use crate::utils::seconds::seconds;
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

impl<R: RealField + FromPrimitive> GalileanSpacetime<R> {
    /// Whether `self` and `other` happen at the same instant: their times are equal once both
    /// are in seconds. Two times counted in a scale that names no duration (`NoScale`, `Steps`,
    /// `Symbolic`) are compared directly when the scales agree; otherwise `None`, the instants
    /// cannot be compared.
    pub fn is_simultaneous_with(&self, other: &Self) -> Option<bool> {
        match (
            seconds(self.t, self.time_scale),
            seconds(other.t, other.time_scale),
        ) {
            (Some(a), Some(b)) => Some(a == b),
            (None, None) if self.time_scale == other.time_scale => Some(self.t == other.t),
            _ => None,
        }
    }
}
