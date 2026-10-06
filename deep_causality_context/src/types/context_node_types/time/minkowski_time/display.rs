/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::MinkowskiTime;
use core::fmt::{Debug, Display};
use deep_causality_algebra::RealField;

impl<R: RealField + Display + Debug> Display for MinkowskiTime<R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "MinkowskiTime: id: {}, time_scale: {}, time_unit: {:?}",
            self.id, self.time_scale, self.time_unit
        )
    }
}
