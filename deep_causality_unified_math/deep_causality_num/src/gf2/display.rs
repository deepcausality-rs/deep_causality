/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::gf2::Gf2;
use core::fmt::{Display, Formatter, Result};

/// Displays the element as the digit it is, so that a printed 𝔽₂ matrix reads as a matrix of
/// zeros and ones rather than of `true` and `false`.
impl Display for Gf2 {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.0 {
            write!(f, "1")
        } else {
            write!(f, "0")
        }
    }
}
