/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Complex, Float};
use core::fmt::{Debug, Formatter};

impl<F> Debug for Complex<F>
where
    F: Float + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "Complex {{ re: {:?}, im: {:?} }}", self.re, self.im)
    }
}
