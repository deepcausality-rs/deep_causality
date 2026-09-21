/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Distance, EuclideanSpacetime};
use deep_causality_algebra::RealField;

impl<R: RealField> Distance for EuclideanSpacetime<R> {
    fn distance(&self, other: &Self) -> R {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
