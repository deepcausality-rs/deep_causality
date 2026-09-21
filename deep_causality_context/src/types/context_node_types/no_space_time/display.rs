/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::NoSpaceTime;
use deep_causality_algebra::RealField;
use std::fmt::{Display, Formatter};

impl<R: RealField> Display for NoSpaceTime<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoSpaceTime")
    }
}
