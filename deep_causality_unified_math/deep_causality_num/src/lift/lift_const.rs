/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The compile-time lifting crossings: a literal into a `const` of the working type.
//!
//! [`lift`](super::lift) runs at the call site, which leaves a program's configuration declared
//! as `const MAX_DEPTH_M: f64 = 30.0` and lifted again at every use. The macros here declare the
//! same constant as `const MAX_DEPTH_M: FloatType = const_scalar_from_int!(FloatType, 30)`, so
//! the alias decides the type and the compiler does the conversion once.
//!
//! | Kind of literal | Macro | Carrier |
//! |---|---|---|
//! | a whole number | [`const_scalar_from_int!`] | `i128` |
//! | a decimal fraction | [`const_scalar_from_float!`] | the `f64` bit pattern |
//!
//! A whole number is exact to 2¹⁰⁶ in `Float106` and to the mantissa elsewhere. A fraction
//! carries the literal's own `f64` value, rounded once per scalar.
//!
//! # Why two macros, and why a macro at all
//!
//! Rust forbids floating-point types as const generic parameters: a const parameter needs
//! structural equality and float equality is not structural, since `NaN != NaN` and `+0.0 ==
//! -0.0`. Integers are allowed, so a whole number travels as an `i128` and a fraction travels as
//! the `u64` bit pattern of its `f64` value. Both are computed in const-generic argument position,
//! which a function cannot do, so the call site is a macro.
//!
//! The split is worth having because the two carriers differ in what they preserve. An `i128`
//! reaches [`Float106`] whole, so `2⁶⁰ + 1` keeps its final bit where an `f64` literal drops it. A
//! fraction has no such route and carries the precision its `f64` literal has.
//!
//! # What these macros are not for
//!
//! A mathematical constant is not a literal. π, ln 2 and e have expansions longer than any literal
//! a source file holds, and a program at [`Float106`] wants thirty-one digits of them. Those come
//! from the scalar itself, as `FloatType::pi()` or `Real::ln(lift::<FloatType>(2.0))`, computed at
//! the working type. Writing one as a literal here would pin it to `f64`'s digits.
//!
//! # Example
//!
//! ```
//! use deep_causality_num::{Float106, const_scalar_from_float, const_scalar_from_int};
//!
//! type FloatType = Float106;
//!
//! const MAX_DEPTH_M: FloatType = const_scalar_from_int!(FloatType, 30);
//! const P_WATER_VAPOUR: FloatType = const_scalar_from_float!(FloatType, 0.0627);
//!
//! // Both are ordinary constants, usable where only a constant is accepted.
//! const BOUNDS: [FloatType; 2] = [MAX_DEPTH_M, P_WATER_VAPOUR];
//! assert_eq!(BOUNDS.len(), 2);
//! ```

use crate::{BFloat16, Float106};

/// A scalar constructible, at compile time, from a whole number.
///
/// The value travels as an `i128`, which every shipped scalar decodes natively. Implement it for a
/// new scalar to give that scalar [`const_scalar_from_int!`].
pub trait ConstScalarFromInt<const N: i128> {
    /// The constant, in this scalar.
    const VALUE: Self;
}

/// A scalar constructible, at compile time, from the bit pattern of an `f64`.
///
/// The value travels as a `u64` because floats cannot be const generic parameters. Implement it
/// for a new scalar to give that scalar [`const_scalar_from_float!`].
pub trait ConstScalarFromFloat<const BITS: u64> {
    /// The constant, in this scalar.
    const VALUE: Self;
}

impl<const N: i128> ConstScalarFromInt<N> for f64 {
    const VALUE: Self = N as f64;
}

impl<const N: i128> ConstScalarFromInt<N> for f32 {
    const VALUE: Self = N as f32;
}

impl<const N: i128> ConstScalarFromInt<N> for BFloat16 {
    const VALUE: Self = BFloat16::round_from_f64(N as f64);
}

impl<const N: i128> ConstScalarFromInt<N> for Float106 {
    /// Two words: the `f64` nearest `N`, then the part it could not hold.
    ///
    /// `hi` is within half an ulp of `N`, so the residue `N - hi` is what the head dropped and the
    /// pair satisfies the `|lo| <= ½ ulp(hi)` invariant. That keeps whole numbers exact past
    /// `f64`'s 2⁵³ ceiling: `2⁶⁰ + 1` comes back with `lo = 1.0`.
    const VALUE: Self = {
        let hi = N as f64;
        // `hi as i128` saturates. The one value where that matters is `hi == 2¹²⁷`, which every
        // `N` from `2¹²⁷ − 2⁷³` up to `i128::MAX` rounds to: the cast returns `i128::MAX`, one
        // short of `hi`, and the residue would then be one too large. Subtracting `i128::MAX`
        // and then `1` reaches `N − 2¹²⁷` without forming `2¹²⁷` in `i128`.
        let residue = if hi >= 170_141_183_460_469_231_731_687_303_715_884_105_728.0 {
            N.wrapping_sub(i128::MAX).wrapping_sub(1)
        } else {
            N.wrapping_sub(hi as i128)
        };
        Float106::from_raw(hi, residue as f64)
    };
}

impl<const BITS: u64> ConstScalarFromFloat<BITS> for f64 {
    const VALUE: Self = f64::from_bits(BITS);
}

impl<const BITS: u64> ConstScalarFromFloat<BITS> for f32 {
    const VALUE: Self = f64::from_bits(BITS) as f32;
}

impl<const BITS: u64> ConstScalarFromFloat<BITS> for BFloat16 {
    const VALUE: Self = BFloat16::round_from_f64(f64::from_bits(BITS));
}

impl<const BITS: u64> ConstScalarFromFloat<BITS> for Float106 {
    /// The head carries the literal's `f64` value and the tail is zero, because an `f64` literal
    /// has nothing further to give. A constant genuinely needing 106 bits is a mathematical
    /// constant, and those come from the scalar rather than from a literal.
    const VALUE: Self = Float106::from_raw(f64::from_bits(BITS), 0.0);
}

/// A whole number as a `const` of the working type.
///
/// `const_scalar_from_int!(FloatType, 30)` is `30` in whatever `FloatType` names, decided by the
/// compiler. The value travels as an `i128`, so negatives work and [`Float106`] stays exact past
/// 2⁵³.
///
/// # Example
///
/// ```
/// use deep_causality_num::{Float106, const_scalar_from_int};
///
/// type FloatType = Float106;
/// const COMPARTMENTS: FloatType = const_scalar_from_int!(FloatType, 16);
/// const BELOW_DATUM: FloatType = const_scalar_from_int!(FloatType, -30);
///
/// assert_eq!(COMPARTMENTS.to_f64(), 16.0);
/// assert_eq!(BELOW_DATUM.to_f64(), -30.0);
/// ```
#[macro_export]
macro_rules! const_scalar_from_int {
    ($t:ty, $v:expr) => {
        <$t as $crate::ConstScalarFromInt<{ $v as i128 }>>::VALUE
    };
}

/// A decimal literal as a `const` of the working type.
///
/// `const_scalar_from_float!(FloatType, 0.0627)` is that literal's `f64` value, rounded once
/// into whatever `FloatType` names. Use [`const_scalar_from_int!`] for a whole number, which is
/// exact at every scalar.
///
/// # Example
///
/// ```
/// use deep_causality_num::{BFloat16, const_scalar_from_float};
///
/// type FloatType = BFloat16;
/// const P_WATER_VAPOUR: FloatType = const_scalar_from_float!(FloatType, 0.0627);
///
/// // Eight significand bits round it to the nearest they can hold.
/// assert_eq!(P_WATER_VAPOUR.to_f64(), 0.0625);
/// ```
#[macro_export]
macro_rules! const_scalar_from_float {
    ($t:ty, $v:expr) => {
        <$t as $crate::ConstScalarFromFloat<{ ($v as f64).to_bits() }>>::VALUE
    };
}
