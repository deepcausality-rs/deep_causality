/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NewtonianTime;
use core::fmt::{Debug, Display};
use deep_causality_algebra::RealField;

impl<R: RealField + Display + Debug> Display for NewtonianTime<R> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "NewtonianTime: id: {}, time_scale: {}, time_unit: {:?}",
            self.id, self.time_scale, self.time_unit
        )
    }
}
