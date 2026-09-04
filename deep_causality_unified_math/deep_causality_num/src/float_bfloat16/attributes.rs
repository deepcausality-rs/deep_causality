/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::BFloat16;

/// The exponent field, all ones for an infinity or a NaN.
const EXPONENT_MASK: u16 = 0x7F80;
/// Everything but the sign.
const MAGNITUDE_MASK: u16 = 0x7FFF;
const SIGN_MASK: u16 = 0x8000;

impl BFloat16 {
    /// Returns `true` if this value is NaN: exponent all ones and a non-zero significand.
    #[inline]
    pub const fn is_nan(self) -> bool {
        (self.bits & MAGNITUDE_MASK) > EXPONENT_MASK
    }

    /// Returns `true` if this value is positive or negative infinity.
    #[inline]
    pub const fn is_infinite(self) -> bool {
        (self.bits & MAGNITUDE_MASK) == EXPONENT_MASK
    }

    /// Returns `true` if this value is neither infinite nor NaN.
    #[inline]
    pub const fn is_finite(self) -> bool {
        (self.bits & EXPONENT_MASK) != EXPONENT_MASK
    }

    /// Returns `true` if the sign bit is clear, which includes `+0.0` and a positive NaN.
    #[inline]
    pub const fn is_sign_positive(self) -> bool {
        (self.bits & SIGN_MASK) == 0
    }

    /// Returns `true` if the sign bit is set, which includes `-0.0` and a negative NaN.
    #[inline]
    pub const fn is_sign_negative(self) -> bool {
        (self.bits & SIGN_MASK) != 0
    }
}
