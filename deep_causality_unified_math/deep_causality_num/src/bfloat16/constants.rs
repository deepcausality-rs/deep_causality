/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::BFloat16;

impl BFloat16 {
    /// The radix of the internal representation.
    pub const RADIX: u32 = 2;

    /// Significant bits, the seven stored plus the implicit leading one.
    pub const MANTISSA_DIGITS: u32 = 8;

    /// Decimal digits that survive a round trip through the type: `floor(7 · log10 2) = 2`.
    pub const DIGITS: u32 = 2;

    /// The difference between `1.0` and the next larger value, `2^-7 = 0.0078125`.
    pub const EPSILON: Self = Self::from_bits(0x3C00);

    /// The smallest finite value, `-(2 - 2^-7) · 2^127 ≈ -3.39e38`.
    pub const MIN: Self = Self::from_bits(0xFF7F);

    /// The smallest positive normal value, `2^-126`, the same as `f32::MIN_POSITIVE`.
    pub const MIN_POSITIVE: Self = Self::from_bits(0x0080);

    /// The largest finite value, `(2 - 2^-7) · 2^127 ≈ 3.39e38`.
    pub const MAX: Self = Self::from_bits(0x7F7F);

    /// One greater than the minimum possible normal power of 2 exponent, as `f32::MIN_EXP`.
    pub const MIN_EXP: i32 = -125;

    /// The maximum possible power of 2 exponent, as `f32::MAX_EXP`.
    pub const MAX_EXP: i32 = 128;

    /// The minimum power of 10 whose value is a normal number.
    pub const MIN_10_EXP: i32 = -37;

    /// The maximum power of 10 whose value is finite.
    pub const MAX_10_EXP: i32 = 38;

    /// The canonical quiet NaN, the top half of `f32::NAN`.
    pub const NAN: Self = Self::from_bits(0x7FC0);

    /// Positive infinity.
    pub const INFINITY: Self = Self::from_bits(0x7F80);

    /// Negative infinity.
    pub const NEG_INFINITY: Self = Self::from_bits(0xFF80);

    /// `+0.0`.
    pub const ZERO: Self = Self::from_bits(0x0000);

    /// `-0.0`.
    pub const NEG_ZERO: Self = Self::from_bits(0x8000);

    /// `1.0`.
    pub const ONE: Self = Self::from_bits(0x3F80);

    /// π rounded to the nearest representable value, `3.140625`.
    pub const PI: Self = Self::round_from_f32(core::f32::consts::PI);

    /// Euler's number rounded to the nearest representable value, `2.71875`.
    pub const E: Self = Self::round_from_f32(core::f32::consts::E);

    /// `ln 2` rounded to the nearest representable value, `0.69140625`.
    pub const LN_2: Self = Self::round_from_f32(core::f32::consts::LN_2);

    /// `ln 10` rounded to the nearest representable value, `2.296875`.
    pub const LN_10: Self = Self::round_from_f32(core::f32::consts::LN_10);
}
