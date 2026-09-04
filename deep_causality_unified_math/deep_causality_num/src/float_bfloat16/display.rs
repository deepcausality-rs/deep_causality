/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::BFloat16;
use core::fmt;
use core::fmt::{LowerExp, UpperExp};

/// Renders the exact value the type holds, through `f32`'s formatting with the caller's
/// formatter, so precision, width, fill, alignment and sign flags all apply.
impl fmt::Display for BFloat16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.to_f32(), f)
    }
}

impl LowerExp for BFloat16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        LowerExp::fmt(&self.to_f32(), f)
    }
}

impl UpperExp for BFloat16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        UpperExp::fmt(&self.to_f32(), f)
    }
}
