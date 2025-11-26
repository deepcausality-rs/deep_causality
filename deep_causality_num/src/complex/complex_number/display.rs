/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Complex, Float};
use core::fmt::{Debug, Display, Formatter};

impl<F> Display for Complex<F>
where
    F: Float + Display + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.im >= F::zero() {
            write!(f, "{}+{}i", self.re, self.im)
        } else {
            write!(f, "{}{:?}i", self.re, self.im)
        }
    }
}
