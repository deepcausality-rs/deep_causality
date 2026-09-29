/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EuclideanSpacetime;
use core::fmt;
use deep_causality_algebra::RealField;

impl<R: RealField + fmt::Display> fmt::Display for EuclideanSpacetime<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EuclideanSpacetime(id={}, x={:.3}, y={:.3}, z={:.3}, t={} {:?})",
            self.id, self.x, self.y, self.z, self.t, self.time_scale
        )
    }
}
