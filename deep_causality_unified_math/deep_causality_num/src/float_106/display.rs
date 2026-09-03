/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Float106;
use core::fmt;
use core::fmt::{LowerExp, UpperExp};

/// Renders the value, honouring the formatter's precision, width, fill, alignment and sign flags.
///
/// A double-double carries about 32 significant digits, and `f64` formatting renders at most the
/// 17 that survive rounding to `f64`. So `Display` shows the value to `f64` precision, which is
/// what a caller writing `{:.3}` or `{:>10}` asks for, and `Debug` shows the two components
/// exactly. The earlier rendering printed `hi+lo` and ignored every flag, which turned `{:.3}`
/// into a surprise at the display boundary.
impl fmt::Display for Float106 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.hi + self.lo;
        fmt::Display::fmt(&value, f)
    }
}

impl LowerExp for Float106 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.hi + self.lo;
        LowerExp::fmt(&value, f)
    }
}

impl UpperExp for Float106 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.hi + self.lo;
        UpperExp::fmt(&value, f)
    }
}
