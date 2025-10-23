/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Complex, Float, ToPrimitive};

impl<F> ToPrimitive for Complex<F>
where
    F: Float + ToPrimitive,
{
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        self.re.to_isize()
    }

    #[inline]
    fn to_i8(&self) -> Option<i8> {
        self.re.to_i8()
    }

    #[inline]
    fn to_i16(&self) -> Option<i16> {
        self.re.to_i16()
    }

    #[inline]
    fn to_i32(&self) -> Option<i32> {
        self.re.to_i32()
    }

    #[inline]
    fn to_i64(&self) -> Option<i64> {
        self.re.to_i64()
    }

    #[inline]
    fn to_i128(&self) -> Option<i128> {
        self.re.to_i128()
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        self.re.to_usize()
    }

    #[inline]
    fn to_u8(&self) -> Option<u8> {
        self.re.to_u8()
    }

    #[inline]
    fn to_u16(&self) -> Option<u16> {
        self.re.to_u16()
    }

    #[inline]
    fn to_u32(&self) -> Option<u32> {
        self.re.to_u32()
    }

    #[inline]
    fn to_u64(&self) -> Option<u64> {
        self.re.to_u64()
    }

    #[inline]
    fn to_u128(&self) -> Option<u128> {
        self.re.to_u128()
    }

    #[inline]
    fn to_f32(&self) -> Option<f32> {
        self.re.to_f32()
    }

    #[inline]
    fn to_f64(&self) -> Option<f64> {
        self.re.to_f64()
    }
}
