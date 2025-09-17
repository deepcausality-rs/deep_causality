/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{FromPrimitive, ToPrimitive};
use std::num::Wrapping;

impl<T: FromPrimitive> FromPrimitive for Wrapping<T> {
    #[inline]
    fn from_isize(n: isize) -> Option<Self> {
        T::from_isize(n).map(Wrapping)
    }
    #[inline]
    fn from_i8(n: i8) -> Option<Self> {
        T::from_i8(n).map(Wrapping)
    }
    #[inline]
    fn from_i16(n: i16) -> Option<Self> {
        T::from_i16(n).map(Wrapping)
    }
    #[inline]
    fn from_i32(n: i32) -> Option<Self> {
        T::from_i32(n).map(Wrapping)
    }
    #[inline]
    fn from_i64(n: i64) -> Option<Self> {
        T::from_i64(n).map(Wrapping)
    }
    #[inline]
    fn from_i128(n: i128) -> Option<Self> {
        T::from_i128(n).map(Wrapping)
    }
    #[inline]
    fn from_usize(n: usize) -> Option<Self> {
        T::from_usize(n).map(Wrapping)
    }
    #[inline]
    fn from_u8(n: u8) -> Option<Self> {
        T::from_u8(n).map(Wrapping)
    }
    #[inline]
    fn from_u16(n: u16) -> Option<Self> {
        T::from_u16(n).map(Wrapping)
    }
    #[inline]
    fn from_u32(n: u32) -> Option<Self> {
        T::from_u32(n).map(Wrapping)
    }
    #[inline]
    fn from_u64(n: u64) -> Option<Self> {
        T::from_u64(n).map(Wrapping)
    }
    #[inline]
    fn from_u128(n: u128) -> Option<Self> {
        T::from_u128(n).map(Wrapping)
    }
    #[inline]
    fn from_f32(n: f32) -> Option<Self> {
        T::from_f32(n).map(Wrapping)
    }
    #[inline]
    fn from_f64(n: f64) -> Option<Self> {
        T::from_f64(n).map(Wrapping)
    }
}

impl<T: ToPrimitive> ToPrimitive for Wrapping<T> {
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        (self.0).to_isize()
    }
    #[inline]
    fn to_i8(&self) -> Option<i8> {
        (self.0).to_i8()
    }
    #[inline]
    fn to_i16(&self) -> Option<i16> {
        (self.0).to_i16()
    }
    #[inline]
    fn to_i32(&self) -> Option<i32> {
        (self.0).to_i32()
    }
    #[inline]
    fn to_i64(&self) -> Option<i64> {
        (self.0).to_i64()
    }
    #[inline]
    fn to_i128(&self) -> Option<i128> {
        (self.0).to_i128()
    }
    #[inline]
    fn to_usize(&self) -> Option<usize> {
        (self.0).to_usize()
    }
    #[inline]
    fn to_u8(&self) -> Option<u8> {
        (self.0).to_u8()
    }
    #[inline]
    fn to_u16(&self) -> Option<u16> {
        (self.0).to_u16()
    }
    #[inline]
    fn to_u32(&self) -> Option<u32> {
        (self.0).to_u32()
    }
    #[inline]
    fn to_u64(&self) -> Option<u64> {
        (self.0).to_u64()
    }
    #[inline]
    fn to_u128(&self) -> Option<u128> {
        (self.0).to_u128()
    }
    #[inline]
    fn to_f32(&self) -> Option<f32> {
        (self.0).to_f32()
    }
    #[inline]
    fn to_f64(&self) -> Option<f64> {
        (self.0).to_f64()
    }
}
