/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Lifting and lowering at the precision boundary.
//!
//! A program that treats precision as a parameter names its working type once, as in
//! `type FloatType = f64`, and writes everything else against a bound. Numbers cross into that
//! type from the primitives a source file can hold, and cross back out for display or to become a
//! count again. These are the crossings, written once, so that no example or crate reinvents them.
//!
//! | Crossing | Function | From | To |
//! |---|---|---|---|
//! | a configuration literal into the working type | [`lift`] | `f64` | `T` |
//! | a count onto the real axis | [`lift_count`] | `u64` | `T` |
//! | any primitive float into the working type | [`lift_f32`], [`lift_f64`] | `f32`, `f64` | `T` |
//! | any primitive integer into the working type | [`lift_i8`] … [`lift_usize`] | `i8` … | `T` |
//! | the display boundary | [`lower`], [`lower_f32`] | `T` | `f64`, `f32` |
//! | a real back to a count, rounded | [`to_count`] | `T` | `u64` |
//! | a whole number into a `const` | [`const_scalar_from_int!`] | `i128` | `T` |
//! | a decimal literal into a `const` | [`const_scalar_from_float!`] | `f64` bits | `T` |
//!
//! The module has two halves. [`lift_try`] holds the runtime crossings, which run at the call site
//! and report a value the target cannot hold. [`lift_const`] holds the compile-time ones, which
//! turn a literal into a `const` already in the working type, so a program's configuration is
//! declared as `const MAX_DEPTH_M: FloatType = const_scalar_from_int!(FloatType, 30)` and never
//! lifted again.
//!
//! Every lift exists for every primitive float (`f32`, `f64`) and every primitive integer (`i8`,
//! `i16`, `i32`, `i64`, `i128`, `isize`, `u8`, `u16`, `u32`, `u64`, `u128`, `usize`), as a
//! `lift_<primitive>` function that panics when the value does not fit and a `try_lift_<primitive>`
//! function that returns `None`. [`lift`] is `lift_f64` under the name the common case deserves,
//! and [`lift_count`] is `lift_u64` likewise.
//!
//! The target is a type parameter and the alias stays where it belongs, in the calling crate:
//! `let x: FloatType = lift(0.5)` resolves `T` from the annotation, and `lift::<FloatType>(0.5)`
//! names it. The [`Lift`] and [`Lower`] traits give the same crossings as methods, so that
//! `0.5.lift()`, `1024u64.lift()` and `x.lower()` read at the call site.
//!
//! # Why not `as`, and why not `From`
//!
//! `as` casts between primitives only, so a program written with `x as FloatType` stops compiling
//! the day the alias becomes `Float106`. `From<f64>` is not implemented for `f32`, so `From`
//! cannot serve the four shipped scalars either. [`FromPrimitive`] and [`ToPrimitive`] are
//! implemented for all of them, and this module is a thin, named layer over those two traits.
//!
//! # What lifting means for precision
//!
//! An `f64` literal is exact in `f64` and in `Float106`, and is rounded into `f32`. That is the
//! point of the parameter: the literal is written once at `f64`, the widest form a source file can
//! hold, and the working type decides what survives. An integer lifts exactly while it fits the
//! target's mantissa and rounds beyond it: values above 2⁵³ round into `f64`, and `Float106`
//! holds every `u64` and `i64` exactly and every integer below 2¹⁰⁶.
//!
//! [`const_scalar_from_int!`] carries a whole number as an `i128` and reaches `Float106` in two
//! words, so `2⁶⁰ + 1` keeps the bit an `f64` literal drops. [`const_scalar_from_float!`] carries
//! the `f64` bit pattern, which is the precision the literal has. A mathematical constant
//! belongs to neither: π
//! and ln 2 run longer than any literal, so they come from the scalar, as `FloatType::pi()` or
//! `Real::ln(lift::<FloatType>(2.0))`, computed at the working type.
//!
//! # Panics
//!
//! The panicking forms panic when the crossing is not representable, because a configuration
//! value that does not fit the working type is a programming error at the boundary, not a runtime
//! condition. The `try_` forms return `Option` for callers that would rather decide.

pub mod lift_const;
pub mod lift_try;

pub use crate::lift::lift_const::{ConstScalarFromFloat, ConstScalarFromInt};
pub use crate::lift::lift_try::{
    Lift, lift, lift_count, lift_f32, lift_f64, lift_i8, lift_i16, lift_i32, lift_i64, lift_i128,
    lift_isize, lift_u8, lift_u16, lift_u32, lift_u64, lift_u128, lift_usize, try_lift,
    try_lift_count, try_lift_f32, try_lift_f64, try_lift_i8, try_lift_i16, try_lift_i32,
    try_lift_i64, try_lift_i128, try_lift_isize, try_lift_u8, try_lift_u16, try_lift_u32,
    try_lift_u64, try_lift_u128, try_lift_usize,
};

use crate::{Float, ToPrimitive};

/// A working-type value lowered to `f64` for display, or `None` if it does not lower.
#[inline]
pub fn try_lower<T: ToPrimitive>(x: T) -> Option<f64> {
    x.to_f64()
}

/// A working-type value lowered to `f64`: the display boundary, and the only place `f64` should
/// appear in a program written against the parameter.
///
/// # Panics
///
/// If `x` does not lower, which no shipped real scalar triggers.
#[inline]
pub fn lower<T: ToPrimitive>(x: T) -> f64 {
    try_lower(x).expect("a working-type value must lower to f64")
}

/// A working-type value lowered to `f32`, or `None` if it does not lower.
#[inline]
pub fn try_lower_f32<T: ToPrimitive>(x: T) -> Option<f32> {
    x.to_f32()
}

/// A working-type value lowered to `f32`, for a consumer that takes single precision.
///
/// # Panics
///
/// If `x` does not lower.
#[inline]
pub fn lower_f32<T: ToPrimitive>(x: T) -> f32 {
    try_lower_f32(x).expect("a working-type value must lower to f32")
}

/// A real rounded back to a count, or `None` if it is not finite, is negative, or does not fit.
///
/// The inverse crossing of [`lift_count`]: a probability times a shot count is a real, and the
/// shots it names are a count again.
#[inline]
pub fn to_count<T: Float + ToPrimitive>(x: T) -> Option<u64> {
    let rounded = x.round();
    if !rounded.is_finite() {
        return None;
    }
    rounded.to_u64()
}

/// The lowering crossings as methods, so that `x.lower()` reads at the display boundary.
/// Implemented for everything that implements [`ToPrimitive`].
pub trait Lower: ToPrimitive + Sized {
    /// To `f64`, or `None` if the value does not lower.
    #[inline]
    fn try_lower(self) -> Option<f64> {
        try_lower(self)
    }

    /// To `f64`.
    ///
    /// # Panics
    ///
    /// If the value does not lower.
    #[inline]
    fn lower(self) -> f64 {
        lower(self)
    }

    /// To `f32`, or `None` if the value does not lower.
    #[inline]
    fn try_lower_f32(self) -> Option<f32> {
        try_lower_f32(self)
    }

    /// To `f32`.
    ///
    /// # Panics
    ///
    /// If the value does not lower.
    #[inline]
    fn lower_f32(self) -> f32 {
        lower_f32(self)
    }
}

impl<T: ToPrimitive> Lower for T {}
