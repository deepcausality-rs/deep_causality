/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::DoubleFloat;
use std::fmt;

impl fmt::Display for DoubleFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // For display, show the sum as best as f64 can represent
        // In practice, users wanting full precision should use hi/lo directly
        if self.lo == 0.0 {
            write!(f, "{}", self.hi)
        } else {
            write!(f, "{}+{}", self.hi, self.lo)
        }
    }
}
