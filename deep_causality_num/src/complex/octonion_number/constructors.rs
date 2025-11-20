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
    /// Creates a new `Octonion` from its eight scalar components.
    ///
    /// # Arguments
    /// * `s` - The scalar (real) part.
    /// * `e1` - The coefficient of the first imaginary unit.
    /// * `e2` - The coefficient of the second imaginary unit.
    /// * `e3` - The coefficient of the third imaginary unit.
    /// * `e4` - The coefficient of the fourth imaginary unit.
    /// * `e5` - The coefficient of the fifth imaginary unit.
    /// * `e6` - The coefficient of the sixth imaginary unit.
    /// * `e7` - The coefficient of the seventh imaginary unit.
    ///
    /// # Returns
    /// A new `Octonion` instance.
    ///
    /// # Examples
    /// ```
    /// use deep_causality_num::Octonion;
    ///
    /// let o = Octonion::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
    /// assert_eq!(o.s, 1.0);
    /// assert_eq!(o.e7, 8.0);
    /// ```
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
    ///
    /// The identity octonion has a scalar part of 1 and all imaginary parts are 0.
    /// When multiplied by any other octonion, it returns the other octonion.
    ///
    /// # Returns
    /// The identity `Octonion` instance.
    ///
    /// # Examples
    /// ```
    /// use deep_causality_num::Octonion;
    /// use deep_causality_num::{One, Zero};
    ///
    /// let identity: Octonion<f64> = Octonion::identity();
    /// assert_eq!(identity.s, 1.0);
    /// assert!(identity.e1.is_zero());
    /// assert!(identity.e7.is_zero());
    ///
    /// // The identity is also accessible via the One trait
    /// assert_eq!(identity, Octonion::one());
    /// ```
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
