/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NoSpaceTime;
use core::fmt::{Display, Formatter};
use deep_causality_algebra::RealField;

impl<R: RealField> Display for NoSpaceTime<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "NoSpaceTime")
    }
}
