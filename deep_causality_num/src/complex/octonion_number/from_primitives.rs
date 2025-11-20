/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::FromPrimitive;
use crate::complex::octonion_number::Octonion;
use crate::float::Float;

/// Implements the `FromPrimitive` trait for `Octonion`.
///
/// This allows various primitive numeric types to be converted into an `Octonion`.
/// The conversion places the primitive value into the scalar (`s`) component of the octonion,
/// with all imaginary components (`e1` through `e7`) set to zero.
impl<F: Float> FromPrimitive for Octonion<F> {
    /// Converts an `isize` value into an `Octonion`.
    /// The `isize` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `isize` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    ///
    /// # Examples
    /// ```
    /// use deep_causality_num::Octonion;
    /// use deep_causality_num::{FromPrimitive, Zero};
    ///
    /// let o = Octonion::<f64>::from_isize(10).unwrap();
    /// assert_eq!(o.s, 10.0);
    /// assert!(o.e1.is_zero());
    /// ```
    fn from_isize(n: isize) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts an `i8` value into an `Octonion`.
    /// The `i8` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `i8` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_i8(n: i8) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts an `i16` value into an `Octonion`.
    /// The `i16` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `i16` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_i16(n: i16) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts an `i32` value into an `Octonion`.
    /// The `i32` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `i32` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_i32(n: i32) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts an `i64` value into an `Octonion`.
    /// The `i64` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `i64` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_i64(n: i64) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts an `i128` value into an `Octonion`.
    /// The `i128` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `i128` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_i128(n: i128) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts a `usize` value into an `Octonion`.
    /// The `usize` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `usize` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_usize(n: usize) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts a `u8` value into an `Octonion`.
    /// The `u8` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `u8` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_u8(n: u8) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts a `u16` value into an `Octonion`.
    /// The `u16` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `u16` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_u16(n: u16) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts a `u32` value into an `Octonion`.
    /// The `u32` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `u32` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_u32(n: u32) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts a `u64` value into an `Octonion`.
    /// The `u64` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `u64` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_u64(n: u64) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts a `u128` value into an `Octonion`.
    /// The `u128` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `u128` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_u128(n: u128) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts an `f32` value into an `Octonion`.
    /// The `f32` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `f32` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_f32(n: f32) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }

    /// Converts an `f64` value into an `Octonion`.
    /// The `f64` value becomes the scalar part.
    ///
    /// # Arguments
    /// * `n` - The `f64` value to convert.
    ///
    /// # Returns
    /// An `Option<Octonion<F>>` containing the new octonion, or `None` if the conversion fails.
    fn from_f64(n: f64) -> Option<Self> {
        Some(Octonion::new(
            F::from(n)?,
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
            F::zero(),
        ))
    }
}
