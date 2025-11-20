/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::Octonion;
use crate::float::Float;
use crate::{NumCast, ToPrimitive};

/// Implements the `NumCast` trait for `Octonion`.
///
/// This enables casting various numeric types into an `Octonion`.
/// The conversion populates the scalar component (`s`) of the octonion,
/// while all imaginary components are set to zero.
///
/// # Type Parameters
/// * `F` - The floating-point type of the octonion's components.
impl<F: Float> NumCast for Octonion<F> {
    /// Converts a value `n` that implements `ToPrimitive` into an `Octonion`.
    ///
    /// # Arguments
    /// * `n` - The value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion of the
    /// scalar component fails.
    ///
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        F::from(n).map(|f| {
            Octonion::new(
                f,
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
                F::zero(),
            )
        })
    }
}
