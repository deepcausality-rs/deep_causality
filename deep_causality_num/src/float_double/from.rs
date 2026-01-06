/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::DoubleFloat;

impl From<f64> for DoubleFloat {
    #[inline]
    fn from(x: f64) -> Self {
        Self::from_f64(x)
    }
}

impl From<f32> for DoubleFloat {
    #[inline]
    fn from(x: f32) -> Self {
        Self::from_f64(x as f64)
    }
}

impl From<i32> for DoubleFloat {
    #[inline]
    fn from(x: i32) -> Self {
        Self::from_f64(x as f64)
    }
}

impl From<i64> for DoubleFloat {
    #[inline]
    fn from(x: i64) -> Self {
        Self::from_f64(x as f64)
    }
}

impl From<u32> for DoubleFloat {
    #[inline]
    fn from(x: u32) -> Self {
        Self::from_f64(x as f64)
    }
}

impl From<u64> for DoubleFloat {
    #[inline]
    fn from(x: u64) -> Self {
        Self::from_f64(x as f64)
    }
}

impl From<DoubleFloat> for f64 {
    #[inline]
    fn from(x: DoubleFloat) -> Self {
        x.hi
    }
}

impl From<DoubleFloat> for f32 {
    #[inline]
    fn from(x: DoubleFloat) -> Self {
        x.hi as f32
    }
}
