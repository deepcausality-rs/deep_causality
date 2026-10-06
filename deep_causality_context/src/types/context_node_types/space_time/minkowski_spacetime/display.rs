/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::MinkowskiSpacetime;
use core::fmt;
use deep_causality_algebra::RealField;

impl<R: RealField + fmt::Display> fmt::Display for MinkowskiSpacetime<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MinkowskiSpacetime(id={}, t={:.3} {:?}, x={:.3}, y={:.3}, z={:.3})",
            self.id, self.t, self.time_scale, self.x, self.y, self.z
        )
    }
}
