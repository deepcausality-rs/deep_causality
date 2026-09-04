/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::BFloat16;

impl BFloat16 {
    /// The representation: sign, 8 exponent bits, 7 significand bits.
    #[inline(always)]
    pub const fn to_bits(self) -> u16 {
        self.bits
    }

    /// The value as an `f32`. Exact: the sixteen low bits of the `f32` are zero.
    #[inline(always)]
    pub const fn to_f32(self) -> f32 {
        f32::from_bits((self.bits as u32) << 16)
    }

    /// The value as an `f64`. Exact.
    #[inline(always)]
    pub const fn to_f64(self) -> f64 {
        self.to_f32() as f64
    }
}
