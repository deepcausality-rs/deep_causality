/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The `bfloat16` floating-point format: the top sixteen bits of an IEEE 754 binary32.
//!
//! | Field | Bits | Same as `f32` |
//! |-------|------|---------------|
//! | sign | 1 | yes |
//! | exponent | 8 | yes, bias 127 |
//! | significand | 7 stored, 8 with the implicit bit | no: `f32` stores 23 |
//!
//! A `BFloat16` therefore holds every `f32` exponent, so its range is that of `f32` (about
//! `1.2e-38` to `3.4e38`, subnormals down to `2^-133`), with two significant decimal digits. It
//! has no IEEE standard of its own; the format is defined by its relationship to binary32, and
//! this type applies binary32's semantics to it: round to nearest with ties to even, subnormals
//! kept, signed zeros, infinities, and NaN with the top payload bits and the sign preserved.
//!
//! # Rounding
//!
//! Every operation returns the correctly rounded result of the exact real operation. The two
//! kernels that make this hold are in this file:
//!
//! * [`BFloat16::round_from_f32`] rounds by adding a bias to the `f32` pattern: the sixteen
//!   discarded bits are compared with the half-way point `0x8000`, and the kept bit decides a tie.
//! * [`BFloat16::round_from_f64`] cannot go through `f32` directly, because two roundings in a
//!   row can land a value that lies just past a `bf16` tie exactly on it, and the tie rule then
//!   sends it the wrong way. It first rounds the `f64` to `f32` under *round to odd*, which keeps
//!   the information that the value was not exactly the tie, and then applies the `f32` kernel.
//!   The intermediate has 24 bits against the target's 8, and round to odd followed by round to
//!   nearest even is correct whenever the intermediate carries at least two more bits than the
//!   target: S. Boldo and G. Melquiond, *Emulation of a FMA and Correctly Rounded Sums: Proved
//!   Algorithms Using Rounding to Odd*, IEEE Transactions on Computers 57(4), 2008.
//!
//! The arithmetic operators compute in `f32` and round once with the first kernel. That is a
//! second rounding after `f32`'s own, and it is harmless here: for `+`, `-`, `*`, `/` and `sqrt`
//! a double rounding gives the correctly rounded result whenever the intermediate precision is at
//! least `2p + 2` for a target of `p` bits, and `24 >= 2 * 8 + 2`: S. A. Figueroa, *When is
//! double rounding innocuous?*, ACM SIGNUM Newsletter 30(3), 1995. Fused multiply-add is not
//! covered by that theorem and gets its own treatment in the `Float` implementation.
//!
//! Integers convert directly from their bits, so a `u64` or `i128` rounds once, not twice.
//!
//! # Where it fits
//!
//! `BFloat16` implements `Float`, so it enters the algebra tower through the same blanket
//! implementations as `f32`, `f64` and `Float106`, and a program written against `FloatType`
//! runs at this precision when the alias names it. The memory footprint is half that of `f32`
//! and the exponent range is identical, which is the trade the format exists for.

mod attributes;
mod constants;
mod debug;
mod display;
mod from;
mod getters;
mod ops_arithmetic;
mod ops_comparison;
mod traits_algebra;
mod traits_num;

/// The bit that makes a NaN quiet: the top bit of the stored significand.
const QUIET_NAN_BIT: u16 = 0x0040;

/// A 16-bit brain floating-point number: sign, 8 exponent bits, 7 stored significand bits.
///
/// The layout is a bare `u16` holding the top half of the equivalent `f32` pattern, so a slice
/// of `BFloat16` is byte-compatible with the `bf16` buffers other libraries and accelerators
/// exchange.
#[derive(Copy, Clone, Default)]
#[repr(transparent)]
pub struct BFloat16 {
    bits: u16,
}

// =============================================================================
// Constructors
// =============================================================================

impl BFloat16 {
    /// The value whose representation is `bits`.
    #[inline]
    pub const fn from_bits(bits: u16) -> Self {
        Self { bits }
    }

    /// The `f32` rounded to the nearest `BFloat16`, ties to even.
    ///
    /// Infinities and signed zeros pass through. A NaN stays a NaN with its sign and the top
    /// seven bits of its significand, and is quieted, so a payload that lived only in the
    /// discarded bits cannot leave an infinity behind.
    #[inline]
    pub const fn round_from_f32(x: f32) -> Self {
        let bits = x.to_bits();
        if x.is_nan() {
            return Self {
                bits: ((bits >> 16) as u16) | QUIET_NAN_BIT,
            };
        }
        // Adding 0x7FFF carries into the kept half exactly when the discarded half exceeds
        // 0x8000; adding the kept half's own low bit turns an exact 0x8000 into a carry only
        // when that bit is odd, which is ties to even. The largest non-NaN pattern is
        // 0xFF80_0000, so the sum cannot overflow a `u32`.
        let kept_lsb = (bits >> 16) & 1;
        Self {
            bits: ((bits + 0x7FFF + kept_lsb) >> 16) as u16,
        }
    }

    /// The `f64` rounded to the nearest `BFloat16`, ties to even, in a single rounding.
    ///
    /// A NaN keeps its sign and the top seven bits of its significand and is quieted. Values
    /// beyond the range round to the infinity of their sign; values below half the smallest
    /// subnormal round to the zero of their sign.
    #[inline]
    pub const fn round_from_f64(x: f64) -> Self {
        if x.is_nan() {
            // The f64 significand occupies bits 51..0 and bf16's occupies bits 6..0, so the
            // seven bits that carry over are 51..45; bit 51 is the quiet bit in both.
            let bits = x.to_bits();
            let sign = ((bits >> 48) as u16) & 0x8000;
            let significand = ((bits >> 45) as u16) & 0x007F;
            return Self {
                bits: sign | 0x7F80 | QUIET_NAN_BIT | significand,
            };
        }
        Self::round_from_f32(round_to_odd_f32(x))
    }

    /// The `f32` as a `BFloat16` if the conversion loses nothing, otherwise `None`.
    ///
    /// The result converts back to the same `f32`. A NaN never compares equal to itself, so it
    /// is not lossless and returns `None`.
    #[inline]
    pub fn from_f32_exact(x: f32) -> Option<Self> {
        if x.is_nan() {
            return None;
        }
        let rounded = Self::round_from_f32(x);
        (rounded.to_f32() == x).then_some(rounded)
    }

    /// The `f64` as a `BFloat16` if the conversion loses nothing, otherwise `None`.
    ///
    /// The result converts back to the same `f64`. A NaN returns `None`.
    #[inline]
    pub fn from_f64_exact(x: f64) -> Option<Self> {
        if x.is_nan() {
            return None;
        }
        let rounded = Self::round_from_f64(x);
        (rounded.to_f64() == x).then_some(rounded)
    }
}

// =============================================================================
// Round to odd
// =============================================================================

/// The `f64` rounded to `f32` under round to odd: the exact value if it is representable,
/// otherwise the neighbouring `f32` whose significand is odd.
///
/// The cast gives the nearest `f32`. When that is not exact and its significand is even, the
/// odd neighbour is the one on the other side of `x`, one pattern away in the direction of `x`.
/// Magnitudes are compared as sign-cleared bit patterns, which order finite and infinite values
/// the way their absolute values do. An overflowing cast yields an infinity with an even
/// significand, and stepping it back gives `f32::MAX`, which is odd; a cast to zero steps up to
/// the smallest subnormal, which is odd. Neither step can wrap, because the sign-cleared pattern
/// of `x` is greater than zero and less than that of infinity in those two cases.
const fn round_to_odd_f32(x: f64) -> f32 {
    let nearest = x as f32;
    if nearest.is_nan() || (nearest as f64) == x {
        return nearest;
    }
    let bits = nearest.to_bits();
    if bits & 1 == 1 {
        return nearest;
    }
    let x_magnitude = x.to_bits() & !(1u64 << 63);
    let nearest_magnitude = (nearest as f64).to_bits() & !(1u64 << 63);
    if x_magnitude > nearest_magnitude {
        f32::from_bits(bits + 1)
    } else {
        f32::from_bits(bits - 1)
    }
}
