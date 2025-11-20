/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ToPrimitive;
use crate::complex::octonion_number::Octonion;
use crate::float::Float;

/// Implements the `ToPrimitive` trait for `Octonion`.
///
/// This allows an `Octonion` to be converted to various primitive numeric types.
/// The conversion is performed on the scalar component (`s`) of the octonion.
/// All `to_` methods return an `Option` to indicate potential conversion failures
/// (e.g., overflow, or if the octonion has non-zero imaginary parts that cannot be
/// represented in the target primitive type, although this implementation only
/// considers the scalar part).
impl<F: Float> ToPrimitive for Octonion<F> {
    /// Attempts to convert the scalar component of the octonion to an `isize`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<isize>` containing the converted value, or `None` if the conversion fails.
    ///
    /// # Examples
    /// ```
    /// use deep_causality_num::{Octonion, ToPrimitive};
    ///
    /// let o = Octonion::new(10.0f64, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    /// assert_eq!(o.to_isize(), Some(10));
    ///
    /// let o_float = Octonion::new(10.5f64, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    /// assert_eq!(o_float.to_isize(), Some(10)); // Truncates
    /// ```
    fn to_isize(&self) -> Option<isize> {
        self.s.to_isize()
    }

    /// Attempts to convert the scalar component of the octonion to an `i8`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<i8>` containing the converted value, or `None` if the conversion fails.
    fn to_i8(&self) -> Option<i8> {
        self.s.to_i8()
    }

    /// Attempts to convert the scalar component of the octonion to an `i16`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<i16>` containing the converted value, or `None` if the conversion fails.
    fn to_i16(&self) -> Option<i16> {
        self.s.to_i16()
    }

    /// Attempts to convert the scalar component of the octonion to an `i32`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<i32>` containing the converted value, or `None` if the conversion fails.
    fn to_i32(&self) -> Option<i32> {
        self.s.to_i32()
    }

    /// Attempts to convert the scalar component of the octonion to an `i64`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<i64>` containing the converted value, or `None` if the conversion fails.
    fn to_i64(&self) -> Option<i64> {
        self.s.to_i64()
    }

    /// Attempts to convert the scalar component of the octonion to an `i128`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<i128>` containing the converted value, or `None` if the conversion fails.
    fn to_i128(&self) -> Option<i128> {
        self.s.to_i128()
    }

    /// Attempts to convert the scalar component of the octonion to a `usize`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<usize>` containing the converted value, or `None` if the conversion fails.
    fn to_usize(&self) -> Option<usize> {
        self.s.to_usize()
    }

    /// Attempts to convert the scalar component of the octonion to a `u8`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<u8>` containing the converted value, or `None` if the conversion fails.
    fn to_u8(&self) -> Option<u8> {
        self.s.to_u8()
    }

    /// Attempts to convert the scalar component of the octonion to a `u16`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<u16>` containing the converted value, or `None` if the conversion fails.
    fn to_u16(&self) -> Option<u16> {
        self.s.to_u16()
    }

    /// Attempts to convert the scalar component of the octonion to a `u32`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<u32>` containing the converted value, or `None` if the conversion fails.
    fn to_u32(&self) -> Option<u32> {
        self.s.to_u32()
    }

    /// Attempts to convert the scalar component of the octonion to a `u64`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<u64>` containing the converted value, or `None` if the conversion fails.
    fn to_u64(&self) -> Option<u64> {
        self.s.to_u64()
    }

    /// Attempts to convert the scalar component of the octonion to a `u128`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<u128>` containing the converted value, or `None` if the conversion fails.
    fn to_u128(&self) -> Option<u128> {
        self.s.to_u128()
    }

    /// Attempts to convert the scalar component of the octonion to an `f32`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<f32>` containing the converted value, or `None` if the conversion fails.
    fn to_f32(&self) -> Option<f32> {
        self.s.to_f32()
    }

    /// Attempts to convert the scalar component of the octonion to an `f64`.
    ///
    /// # Arguments
    /// * `self` - The `Octonion` to convert.
    ///
    /// # Returns
    /// An `Option<f64>` containing the converted value, or `None` if the conversion fails.
    fn to_f64(&self) -> Option<f64> {
        self.s.to_f64()
    }
}
