/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::complex::octonion_number::Octonion;
use crate::float::Float;
use crate::{AsPrimitive, NumCast};

/// Implements the `AsPrimitive` trait for `Octonion`.
///
/// This allows an `Octonion` to be converted to a primitive type `T`.
/// The conversion is performed on the scalar component (`s`) of the octonion.
///
/// # Type Parameters
/// * `F` - The floating-point type of the octonion's components.
/// * `T` - The target primitive type to convert to.
///
/// # Arguments
/// * `self` - The `Octonion` to convert.
///
/// # Returns
/// The scalar component `s` of the octonion converted to type `T`.
///
/// # Examples
/// ```
/// use deep_causality_num::Octonion;
/// use deep_causality_num::AsPrimitive;
///
/// let o = Octonion::new(42.5f64, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
/// let i: i32 = o.as_();
/// assert_eq!(i, 42);
///
/// let f: f32 = o.as_();
/// assert_eq!(f, 42.5f32);
/// ```
impl<F: Float, T> AsPrimitive<T> for Octonion<F>
where
    F: AsPrimitive<T>,
    T: 'static + Copy + NumCast,
{
    fn as_(self) -> T {
        self.s.as_()
    }
}
