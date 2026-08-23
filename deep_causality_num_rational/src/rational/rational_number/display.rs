/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use super::{Rational, RationalScalar};
use core::fmt::{Display, Formatter, Result};

impl<T: RationalScalar + Display> Display for Rational<T> {
    /// Renders `n/d`, or just `n` when the denominator is one.
    ///
    /// ```
    /// use deep_causality_num_rational::Rational;
    ///
    /// assert_eq!(Rational::new(3_i64, 4).to_string(), "3/4");
    /// assert_eq!(Rational::new(6_i64, 3).to_string(), "2");
    /// assert_eq!(Rational::new(1_i64, -2).to_string(), "-1/2");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if *self.denom() == T::one() {
            write!(f, "{}", self.numer())
        } else {
            write!(f, "{}/{}", self.numer(), self.denom())
        }
    }
}
