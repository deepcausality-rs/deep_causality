/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::EcefSpace;
use core::fmt;
use deep_causality_algebra::RealField;

impl<R: RealField + fmt::Display> fmt::Display for EcefSpace<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EcefSpace(id={}, x={:.4}, y={:.4}, z={:.4})",
            self.id, self.x, self.y, self.z
        )
    }
}
