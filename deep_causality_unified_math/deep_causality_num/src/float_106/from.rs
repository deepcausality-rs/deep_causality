/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Float106;

/// The single `f64` constructor: the low component is zero.
///
/// This is the one way to build a `Float106` from an `f64` value. An inherent `from_f64` used to
/// exist beside it and shadowed `FromPrimitive::from_f64`, returning a value where the trait
/// returns an `Option`; callers writing the trait method got the inherent one, and the failure
/// was a type error at the wrong line. It is retired, and `From` is what remains.
impl From<f64> for Float106 {
    #[inline]
    fn from(x: f64) -> Self {
        Self { hi: x, lo: 0.0 }
    }
}

impl From<f32> for Float106 {
    #[inline]
    fn from(x: f32) -> Self {
        Self::from(x as f64)
    }
}

impl From<i32> for Float106 {
    #[inline]
    fn from(x: i32) -> Self {
        Self::from(x as f64)
    }
}

impl From<i64> for Float106 {
    #[inline]
    fn from(x: i64) -> Self {
        Self::from(x as f64)
    }
}

impl From<u32> for Float106 {
    #[inline]
    fn from(x: u32) -> Self {
        Self::from(x as f64)
    }
}

impl From<u64> for Float106 {
    #[inline]
    fn from(x: u64) -> Self {
        Self::from(x as f64)
    }
}

impl From<Float106> for f64 {
    #[inline]
    fn from(x: Float106) -> Self {
        x.hi
    }
}

impl From<Float106> for f32 {
    #[inline]
    fn from(x: Float106) -> Self {
        x.hi as f32
    }
}
