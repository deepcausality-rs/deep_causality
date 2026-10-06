/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Distance, GalileanSpacetime};
use deep_causality_algebra::RealField;
use deep_causality_num::FromPrimitive;

impl<R: RealField + FromPrimitive> Distance for GalileanSpacetime<R> {
    /// `√(Δx² + Δy² + Δz²)` for two simultaneous events, and NaN for any other pair, between
    /// which Galilean spacetime defines no spatial distance. See
    /// [`is_simultaneous_with`](GalileanSpacetime::is_simultaneous_with).
    fn distance(&self, other: &Self) -> R {
        if self.is_simultaneous_with(other) != Some(true) {
            return R::nan();
        }
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
