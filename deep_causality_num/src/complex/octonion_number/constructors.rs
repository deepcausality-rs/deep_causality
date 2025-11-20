/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::Octonion;
use crate::float::Float;

impl<F> Octonion<F>
where
    F: Float,
{
    /// Creates a new octonion from its eight components.
    #[allow(clippy::too_many_arguments)]
    pub fn new(s: F, e1: F, e2: F, e3: F, e4: F, e5: F, e6: F, e7: F) -> Self {
        Self {
            s,
            e1,
            e2,
            e3,
            e4,
            e5,
            e6,
            e7,
        }
    }

    /// Returns the identity octonion (1 + 0e₁ + ... + 0e₇).
    pub fn identity() -> Self {
        Self {
            s: F::one(),
            e1: F::zero(),
            e2: F::zero(),
            e3: F::zero(),
            e4: F::zero(),
            e5: F::zero(),
            e6: F::zero(),
            e7: F::zero(),
        }
    }
}
