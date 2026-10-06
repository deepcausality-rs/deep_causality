/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Distance, NewtonianSpacetime};
use deep_causality_algebra::RealField;

impl<R: RealField> Distance for NewtonianSpacetime<R> {
    /// The distance in absolute space, `√(Δx² + Δy² + Δz²)`, defined for any two events. It is
    /// zero for two events at the same place at different times, so it is a pseudometric on
    /// events.
    fn distance(&self, other: &Self) -> R {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
