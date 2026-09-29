/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EuclideanSpace;
use core::fmt::{Display, Formatter};
use deep_causality_algebra::RealField;

impl<R: RealField + Display> Display for EuclideanSpace<R> {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(
            f,
            "EuclideanSpace(id={}, x={:.4}, y={:.4}, z={:.4})",
            self.id, self.x, self.y, self.z
        )
    }
}
