/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::Octonion;
use crate::float::Float;
use std::fmt::Display;

// Display
impl<F: Float + Display> Display for Octonion<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.s)?;
        write!(
            f,
            " {} {}e₁",
            if self.e1 < F::zero() { "-" } else { "+" },
            self.e1.abs()
        )?;
        write!(
            f,
            " {} {}e₂",
            if self.e2 < F::zero() { "-" } else { "+" },
            self.e2.abs()
        )?;
        write!(
            f,
            " {} {}e₃",
            if self.e3 < F::zero() { "-" } else { "+" },
            self.e3.abs()
        )?;
        write!(
            f,
            " {} {}e₄",
            if self.e4 < F::zero() { "-" } else { "+" },
            self.e4.abs()
        )?;
        write!(
            f,
            " {} {}e₅",
            if self.e5 < F::zero() { "-" } else { "+" },
            self.e5.abs()
        )?;
        write!(
            f,
            " {} {}e₆",
            if self.e6 < F::zero() { "-" } else { "+" },
            self.e6.abs()
        )?;
        write!(
            f,
            " {} {}e₇",
            if self.e7 < F::zero() { "-" } else { "+" },
            self.e7.abs()
        )
    }
}
