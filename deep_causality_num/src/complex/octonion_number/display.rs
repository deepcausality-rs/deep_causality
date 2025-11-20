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
        let mut is_first_term = true;

        if !self.s.is_zero() {
            write!(f, "{}", self.s)?;
            is_first_term = false;
        }

        let components = [
            (self.e1, "e₁"),
            (self.e2, "e₂"),
            (self.e3, "e₃"),
            (self.e4, "e₄"),
            (self.e5, "e₅"),
            (self.e6, "e₆"),
            (self.e7, "e₇"),
        ];

        for (value, symbol) in components.iter() {
            if !value.is_zero() {
                if is_first_term {
                    if value.is_sign_negative() {
                        write!(f, "-{}{}", value.abs(), symbol)?;
                    } else {
                        write!(f, "{}{}", value.abs(), symbol)?;
                    }
                    is_first_term = false;
                } else if value.is_sign_negative() {
                    write!(f, " - {}{}", value.abs(), symbol)?;
                } else {
                    write!(f, " + {}{}", value.abs(), symbol)?;
                }
            }
        }

        if is_first_term {
            // Means nothing was printed, so it's a zero octonion
            write!(f, "0")
        } else {
            Ok(())
        }
    }
}
