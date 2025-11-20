/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use std::ops::Neg;

use crate::complex::octonion_number::Octonion;
use crate::float::Float;

// Neg
impl<F: Float> Neg for Octonion<F> {
    type Output = Self;
    fn neg(self) -> Self {
        Octonion {
            s: -self.s,
            e1: -self.e1,
            e2: -self.e2,
            e3: -self.e3,
            e4: -self.e4,
            e5: -self.e5,
            e6: -self.e6,
            e7: -self.e7,
        }
    }
}
