/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Complex, Float};

impl<F> Complex<F>
where
    F: Float,
{
    #[inline]
    pub fn new(re: F, im: F) -> Self {
        Self { re, im }
    }

    #[inline]
    pub fn from_real(re: F) -> Self {
        Self { re, im: F::zero() }
    }
}
