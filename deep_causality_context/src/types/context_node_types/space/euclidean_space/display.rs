/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EuclideanSpace;
use deep_causality_algebra::RealField;
use std::fmt::{Display, Formatter};

impl<R: RealField + Display> Display for EuclideanSpace<R> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "EuclideanSpace(id={}, x={:.4}, y={:.4}, z={:.4})",
            self.id, self.x, self.y, self.z
        )
    }
}
