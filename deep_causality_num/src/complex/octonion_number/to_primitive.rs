/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ToPrimitive;
use crate::complex::octonion_number::Octonion;
use crate::float::Float;

impl<F: Float> ToPrimitive for Octonion<F> {
    fn to_isize(&self) -> Option<isize> {
        self.s.to_isize()
    }

    fn to_i8(&self) -> Option<i8> {
        self.s.to_i8()
    }

    fn to_i16(&self) -> Option<i16> {
        self.s.to_i16()
    }

    fn to_i32(&self) -> Option<i32> {
        self.s.to_i32()
    }

    fn to_i64(&self) -> Option<i64> {
        self.s.to_i64()
    }

    fn to_i128(&self) -> Option<i128> {
        self.s.to_i128()
    }

    fn to_usize(&self) -> Option<usize> {
        self.s.to_usize()
    }

    fn to_u8(&self) -> Option<u8> {
        self.s.to_u8()
    }

    fn to_u16(&self) -> Option<u16> {
        self.s.to_u16()
    }

    fn to_u32(&self) -> Option<u32> {
        self.s.to_u32()
    }

    fn to_u64(&self) -> Option<u64> {
        self.s.to_u64()
    }

    fn to_u128(&self) -> Option<u128> {
        self.s.to_u128()
    }

    fn to_f32(&self) -> Option<f32> {
        self.s.to_f32()
    }

    fn to_f64(&self) -> Option<f64> {
        self.s.to_f64()
    }
}
