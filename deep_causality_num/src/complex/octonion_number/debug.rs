/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::Octonion;
use crate::float::Float;
use std::fmt::Debug;

// Debug
impl<F: Float + Debug> Debug for Octonion<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Octonion")
            .field("s", &self.s)
            .field("e1", &self.e1)
            .field("e2", &self.e2)
            .field("e3", &self.e3)
            .field("e4", &self.e4)
            .field("e5", &self.e5)
            .field("e6", &self.e6)
            .field("e7", &self.e7)
            .finish()
    }
}
