/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::cast::to_primitive::ToPrimitive;
use std::num::Wrapping;

/// A generic trait for converting a number to a value.
///
/// A value can be represented by the target type when it lies within
/// the range of scalars supported by the target type.
/// For example, a negative integer cannot be represented by an unsigned
/// integer type, and an `i64` with a very high magnitude might not be
/// convertible to an `i32`.
/// On the other hand, conversions with possible precision loss or truncation
/// are admitted, like an `f32` with a decimal part to an integer type, or
/// even a large `f64` saturating to `f32` infinity.
pub trait FromPrimitive: Sized {
    /// Converts an `isize` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    #[inline]
    fn from_isize(n: isize) -> Option<Self> {
        n.to_i64().and_then(FromPrimitive::from_i64)
    }

    /// Converts an `i8` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    #[inline]
    fn from_i8(n: i8) -> Option<Self> {
        FromPrimitive::from_i64(From::from(n))
    }

    /// Converts an `i16` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    #[inline]
    fn from_i16(n: i16) -> Option<Self> {
        FromPrimitive::from_i64(From::from(n))
    }

    /// Converts an `i32` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    #[inline]
    fn from_i32(n: i32) -> Option<Self> {
        FromPrimitive::from_i64(From::from(n))
    }

    /// Converts an `i64` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_i64(n: i64) -> Option<Self>;

    /// Converts an `i128` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    ///
    /// The default implementation converts through `from_i64()`. Types implementing
    /// this trait should override this method if they can represent a greater range.
    #[inline]
    fn from_i128(n: i128) -> Option<Self> {
        n.to_i64().and_then(FromPrimitive::from_i64)
    }

    /// Converts a `usize` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    #[inline]
    fn from_usize(n: usize) -> Option<Self> {
        n.to_u64().and_then(FromPrimitive::from_u64)
    }

    /// Converts an `u8` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    #[inline]
    fn from_u8(n: u8) -> Option<Self> {
        FromPrimitive::from_u64(From::from(n))
    }

    /// Converts an `u16` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    #[inline]
    fn from_u16(n: u16) -> Option<Self> {
        FromPrimitive::from_u64(From::from(n))
    }

    /// Converts an `u32` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    #[inline]
    fn from_u32(n: u32) -> Option<Self> {
        FromPrimitive::from_u64(From::from(n))
    }

    /// Converts an `u64` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    fn from_u64(n: u64) -> Option<Self>;

    /// Converts an `u128` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    ///
    /// The default implementation converts through `from_u64()`. Types implementing
    /// this trait should override this method if they can represent a greater range.
    #[inline]
    fn from_u128(n: u128) -> Option<Self> {
        n.to_u64().and_then(FromPrimitive::from_u64)
    }

    /// Converts a `f32` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    #[inline]
    fn from_f32(n: f32) -> Option<Self> {
        FromPrimitive::from_f64(From::from(n))
    }

    /// Converts a `f64` to return an optional value of this type. If the
    /// value cannot be represented by this type, then `None` is returned.
    ///
    /// The default implementation tries to convert through `from_i64()`, and
    /// failing that through `from_u64()`. Types implementing this trait should
    /// override this method if they can represent a greater range.
    #[inline]
    fn from_f64(n: f64) -> Option<Self> {
        match n.to_i64() {
            Some(i) => FromPrimitive::from_i64(i),
            None => n.to_u64().and_then(FromPrimitive::from_u64),
        }
    }
}

macro_rules! impl_from_primitive {
    ($T:ty, $to_ty:ident) => {
        #[allow(deprecated)]
        impl FromPrimitive for $T {
            #[inline]
            fn from_isize(n: isize) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i8(n: i8) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i16(n: i16) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i32(n: i32) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i64(n: i64) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_i128(n: i128) -> Option<$T> {
                n.$to_ty()
            }

            #[inline]
            fn from_usize(n: usize) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u8(n: u8) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u16(n: u16) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u32(n: u32) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u64(n: u64) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_u128(n: u128) -> Option<$T> {
                n.$to_ty()
            }

            #[inline]
            fn from_f32(n: f32) -> Option<$T> {
                n.$to_ty()
            }
            #[inline]
            fn from_f64(n: f64) -> Option<$T> {
                n.$to_ty()
            }
        }
    };
}

impl_from_primitive!(isize, to_isize);
impl_from_primitive!(i8, to_i8);
impl_from_primitive!(i16, to_i16);
impl_from_primitive!(i32, to_i32);
impl_from_primitive!(i64, to_i64);
impl_from_primitive!(i128, to_i128);
impl_from_primitive!(usize, to_usize);
impl_from_primitive!(u8, to_u8);
impl_from_primitive!(u16, to_u16);
impl_from_primitive!(u32, to_u32);
impl_from_primitive!(u64, to_u64);
impl_from_primitive!(u128, to_u128);
impl_from_primitive!(f32, to_f32);
impl_from_primitive!(f64, to_f64);

macro_rules! impl_to_primitive_wrapping {
    ($( $(#[$cfg:meta])* fn $method:ident -> $i:ident ; )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(&self) -> Option<$i> {
            (self.0).$method()
        }
    )*}
}

impl<T: ToPrimitive> ToPrimitive for Wrapping<T> {
    impl_to_primitive_wrapping! {
        fn to_isize -> isize;
        fn to_i8 -> i8;
        fn to_i16 -> i16;
        fn to_i32 -> i32;
        fn to_i64 -> i64;
        fn to_i128 -> i128;

        fn to_usize -> usize;
        fn to_u8 -> u8;
        fn to_u16 -> u16;
        fn to_u32 -> u32;
        fn to_u64 -> u64;
        fn to_u128 -> u128;

        fn to_f32 -> f32;
        fn to_f64 -> f64;
    }
}

macro_rules! impl_from_primitive_wrapping {
    ($( $(#[$cfg:meta])* fn $method:ident ( $i:ident ); )*) => {$(
        #[inline]
        $(#[$cfg])*
        fn $method(n: $i) -> Option<Self> {
            T::$method(n).map(Wrapping)
        }
    )*}
}

impl<T: FromPrimitive> FromPrimitive for Wrapping<T> {
    impl_from_primitive_wrapping! {
        fn from_isize(isize);
        fn from_i8(i8);
        fn from_i16(i16);
        fn from_i32(i32);
        fn from_i64(i64);
        fn from_i128(i128);

        fn from_usize(usize);
        fn from_u8(u8);
        fn from_u16(u16);
        fn from_u32(u32);
        fn from_u64(u64);
        fn from_u128(u128);

        fn from_f32(f32);
        fn from_f64(f64);
    }
}
