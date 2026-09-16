/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The runtime lifting crossings: a primitive into the working type, checked or panicking.
//!
//! Every lift here goes through [`FromPrimitive`], so it runs at the call site and reports a value
//! the target cannot hold. The compile-time counterparts live in
//! [`lift_const`](super::lift_const), which turn a literal into a `const` of the working type.

use crate::FromPrimitive;

/// The lifting crossings as methods on the source, so that `0.5.lift()` and `1024u64.lift()`
/// read at the call site. Implemented for every primitive float and integer.
pub trait Lift: Sized {
    /// Into the working type, or `None` if `T` cannot represent the value.
    fn try_lift<T: FromPrimitive>(self) -> Option<T>;

    /// Into the working type.
    ///
    /// # Panics
    ///
    /// If `T` cannot represent the value.
    fn lift<T: FromPrimitive>(self) -> T {
        self.try_lift()
            .expect("a value must be representable in the working type")
    }
}

/// One lift per primitive: the `Lift` impl, the `try_` function and the panicking function.
macro_rules! lift_from {
    ($($src:ty => $via:ident, $try_fn:ident, $lift_fn:ident;)*) => {
        $(
            impl Lift for $src {
                #[inline]
                fn try_lift<T: FromPrimitive>(self) -> Option<T> {
                    T::$via(self)
                }
            }

            #[doc = concat!(
                "A `", stringify!($src),
                "` lifted into the working type, or `None` if `T` cannot represent it."
            )]
            #[inline]
            pub fn $try_fn<T: FromPrimitive>(x: $src) -> Option<T> {
                T::$via(x)
            }

            #[doc = concat!("A `", stringify!($src), "` lifted into the working type.")]
            ///
            /// # Panics
            ///
            /// If `T` cannot represent `x`.
            #[inline]
            pub fn $lift_fn<T: FromPrimitive>(x: $src) -> T {
                $try_fn(x).expect(concat!(
                    "a ", stringify!($src), " must be representable in the working type"
                ))
            }
        )*
    };
}

lift_from! {
    f32 => from_f32, try_lift_f32, lift_f32;
    f64 => from_f64, try_lift_f64, lift_f64;
    i8 => from_i8, try_lift_i8, lift_i8;
    i16 => from_i16, try_lift_i16, lift_i16;
    i32 => from_i32, try_lift_i32, lift_i32;
    i64 => from_i64, try_lift_i64, lift_i64;
    i128 => from_i128, try_lift_i128, lift_i128;
    isize => from_isize, try_lift_isize, lift_isize;
    u8 => from_u8, try_lift_u8, lift_u8;
    u16 => from_u16, try_lift_u16, lift_u16;
    u32 => from_u32, try_lift_u32, lift_u32;
    u64 => from_u64, try_lift_u64, lift_u64;
    u128 => from_u128, try_lift_u128, lift_u128;
    usize => from_usize, try_lift_usize, lift_usize;
}

/// A configuration literal lifted into the working type, or `None` if `T` cannot represent it.
///
/// The same crossing as [`try_lift_f64`], under the name the common case deserves.
#[inline]
pub fn try_lift<T: FromPrimitive>(x: f64) -> Option<T> {
    try_lift_f64(x)
}

/// A configuration literal lifted into the working type.
///
/// The same crossing as [`lift_f64`], under the name the common case deserves: a literal is
/// written once at `f64`, the widest form a source file can hold, and lifted where it is used.
///
/// # Panics
///
/// If `T` cannot represent `x`.
#[inline]
pub fn lift<T: FromPrimitive>(x: f64) -> T {
    try_lift(x).expect("a configuration literal must be representable in the working type")
}

/// A count lifted onto the real axis, or `None` if `T` cannot represent it.
///
/// The same crossing as [`try_lift_u64`], under the name the common case deserves.
#[inline]
pub fn try_lift_count<T: FromPrimitive>(n: u64) -> Option<T> {
    try_lift_u64(n)
}

/// A count lifted onto the real axis: a shot count, a dimension, a step index.
///
/// The same crossing as [`lift_u64`], under the name the common case deserves.
///
/// # Panics
///
/// If `T` cannot represent `n`.
#[inline]
pub fn lift_count<T: FromPrimitive>(n: u64) -> T {
    try_lift_count(n).expect("a count must be representable in the working type")
}
