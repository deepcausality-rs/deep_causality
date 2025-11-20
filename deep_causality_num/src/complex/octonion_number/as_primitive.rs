/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::Octonion;
use crate::float::Float;
use crate::{AsPrimitive, NumCast};

// AsPrimitive
impl<F: Float, T> AsPrimitive<T> for Octonion<F>
where
    F: AsPrimitive<T>,
    T: 'static + Copy + NumCast,
{
    fn as_(self) -> T {
        self.s.as_()
    }
}
