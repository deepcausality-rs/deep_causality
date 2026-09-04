/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::BFloat16;

/// Rounds to the nearest representable value, ties to even.
///
/// `From` rounds here for the same reason a literal written at `f64` rounds into `f32`: the
/// working type decides what survives the crossing. `From<f64>` on `Float106` is the widening
/// counterpart. A caller that needs to know whether anything was lost has
/// [`BFloat16::from_f32_exact`].
impl From<f32> for BFloat16 {
    #[inline]
    fn from(x: f32) -> Self {
        Self::round_from_f32(x)
    }
}

/// Rounds to the nearest representable value, ties to even, in a single rounding.
///
/// A caller that needs to know whether anything was lost has [`BFloat16::from_f64_exact`].
impl From<f64> for BFloat16 {
    #[inline]
    fn from(x: f64) -> Self {
        Self::round_from_f64(x)
    }
}

/// Exact.
impl From<BFloat16> for f32 {
    #[inline]
    fn from(x: BFloat16) -> Self {
        x.to_f32()
    }
}

/// Exact.
impl From<BFloat16> for f64 {
    #[inline]
    fn from(x: BFloat16) -> Self {
        x.to_f64()
    }
}
