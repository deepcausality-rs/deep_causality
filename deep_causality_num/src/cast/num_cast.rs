/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::ToPrimitive;

/// Cast from one machine scalar to another.
#[inline]
pub fn cast<T: NumCast, U: NumCast>(n: T) -> Option<U> {
    NumCast::from(n)
}

/// An interface for casting between machine scalars.
pub trait NumCast: Sized + ToPrimitive {
    /// Creates a number from another value that can be converted into
    /// a primitive via the `ToPrimitive` trait. If the source value cannot be
    /// represented by the target type, then `None` is returned.
    ///
    /// A value can be represented by the target type when it lies within
    /// the range of scalars supported by the target type.
    /// For example, a negative integer cannot be represented by an unsigned
    /// integer type, and an `i64` with a very high magnitude might not be
    /// convertible to an `i32`.
    /// On the other hand, conversions with possible precision loss or truncation
    /// are admitted, like an `f32` with a decimal part to an integer type, or
    /// even a large `f64` saturating to `f32` infinity.
    fn from<T: ToPrimitive>(n: T) -> Option<Self>;
}

macro_rules! impl_num_cast {
    ($T:ty, $conv:ident) => {
        impl NumCast for $T {
            #[inline]
            #[allow(deprecated)]
            fn from<N: ToPrimitive>(n: N) -> Option<$T> {
                // `$conv` could be generated using `concat_idents!`, but that
                // macro seems to be broken at the moment
                n.$conv()
            }
        }
    };
}

impl_num_cast!(u8, to_u8);
impl_num_cast!(u16, to_u16);
impl_num_cast!(u32, to_u32);
impl_num_cast!(u64, to_u64);
impl_num_cast!(u128, to_u128);
impl_num_cast!(usize, to_usize);
impl_num_cast!(i8, to_i8);
impl_num_cast!(i16, to_i16);
impl_num_cast!(i32, to_i32);
impl_num_cast!(i64, to_i64);
impl_num_cast!(i128, to_i128);
impl_num_cast!(isize, to_isize);
impl_num_cast!(f32, to_f32);
impl_num_cast!(f64, to_f64);
