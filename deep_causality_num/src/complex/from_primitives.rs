/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{Complex, Float, FromPrimitive};

impl<F> FromPrimitive for Complex<F>
where
    F: Float + FromPrimitive,
{
    #[inline]
    fn from_isize(n: isize) -> Option<Self> {
        F::from_isize(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_i8(n: i8) -> Option<Self> {
        F::from_i8(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_i16(n: i16) -> Option<Self> {
        F::from_i16(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_i32(n: i32) -> Option<Self> {
        F::from_i32(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_i64(n: i64) -> Option<Self> {
        F::from_i64(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_i128(n: i128) -> Option<Self> {
        F::from_i128(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_usize(n: usize) -> Option<Self> {
        F::from_usize(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_u8(n: u8) -> Option<Self> {
        F::from_u8(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_u16(n: u16) -> Option<Self> {
        F::from_u16(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_u32(n: u32) -> Option<Self> {
        F::from_u32(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_u64(n: u64) -> Option<Self> {
        F::from_u64(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_u128(n: u128) -> Option<Self> {
        F::from_u128(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_f32(n: f32) -> Option<Self> {
        F::from_f32(n).map(|re| Self::new(re, F::zero()))
    }

    #[inline]
    fn from_f64(n: f64) -> Option<Self> {
        F::from_f64(n).map(|re| Self::new(re, F::zero()))
    }
}
