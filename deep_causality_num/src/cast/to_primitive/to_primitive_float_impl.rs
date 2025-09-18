/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::ToPrimitive;
use core::mem::size_of;
use core::{f32, f64};

impl ToPrimitive for f32 {
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        if size_of::<f32>() > size_of::<isize>() {
            const MIN_M1: f32 = isize::MIN as f32 - 1.0;
            const MAX_P1: f32 = isize::MAX as f32 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as isize);
            }
        } else {
            const MIN: f32 = isize::MIN as f32;
            const MAX_P1: f32 = isize::MAX as f32;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as isize);
            }
        }
        None
    }

    #[inline]
    fn to_i8(&self) -> Option<i8> {
        if size_of::<f32>() > size_of::<i8>() {
            const MIN_M1: f32 = i8::MIN as f32 - 1.0;
            const MAX_P1: f32 = i8::MAX as f32 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as i8);
            }
        } else {
            const MIN: f32 = i8::MIN as f32;
            const MAX_P1: f32 = i8::MAX as f32;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as i8);
            }
        }
        None
    }

    #[inline]
    fn to_i16(&self) -> Option<i16> {
        if size_of::<f32>() > size_of::<i16>() {
            const MIN_M1: f32 = i16::MIN as f32 - 1.0;
            const MAX_P1: f32 = i16::MAX as f32 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as i16);
            }
        } else {
            const MIN: f32 = i16::MIN as f32;
            const MAX_P1: f32 = i16::MAX as f32;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as i16);
            }
        }
        None
    }

    #[inline]
    fn to_i32(&self) -> Option<i32> {
        if size_of::<f32>() > size_of::<i32>() {
            const MIN_M1: f32 = i32::MIN as f32 - 1.0;
            const MAX_P1: f32 = i32::MAX as f32 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as i32);
            }
        } else {
            const MIN: f32 = i32::MIN as f32;
            const MAX_P1: f32 = i32::MAX as f32;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as i32);
            }
        }
        None
    }

    #[inline]
    fn to_i64(&self) -> Option<i64> {
        if size_of::<f32>() > size_of::<i64>() {
            const MIN_M1: f32 = i64::MIN as f32 - 1.0;
            const MAX_P1: f32 = i64::MAX as f32 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as i64);
            }
        } else {
            const MIN: f32 = i64::MIN as f32;
            const MAX_P1: f32 = i64::MAX as f32;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as i64);
            }
        }
        None
    }

    #[inline]
    fn to_i128(&self) -> Option<i128> {
        if size_of::<f32>() > size_of::<i128>() {
            const MIN_M1: f32 = i128::MIN as f32 - 1.0;
            const MAX_P1: f32 = i128::MAX as f32 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as i128);
            }
        } else {
            const MIN: f32 = i128::MIN as f32;
            const MAX_P1: f32 = i128::MAX as f32;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as i128);
            }
        }
        None
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        if size_of::<f32>() > size_of::<usize>() {
            const MAX_P1: f32 = usize::MAX as f32 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as usize);
            }
        } else {
            const MAX_P1: f32 = usize::MAX as f32;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as usize);
            }
        }
        None
    }

    #[inline]
    fn to_u8(&self) -> Option<u8> {
        if size_of::<f32>() > size_of::<u8>() {
            const MAX_P1: f32 = u8::MAX as f32 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u8);
            }
        } else {
            const MAX_P1: f32 = u8::MAX as f32;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u8);
            }
        }
        None
    }

    #[inline]
    fn to_u16(&self) -> Option<u16> {
        if size_of::<f32>() > size_of::<u16>() {
            const MAX_P1: f32 = u16::MAX as f32 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u16);
            }
        } else {
            const MAX_P1: f32 = u16::MAX as f32;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u16);
            }
        }
        None
    }

    #[inline]
    fn to_u32(&self) -> Option<u32> {
        if size_of::<f32>() > size_of::<u32>() {
            const MAX_P1: f32 = u32::MAX as f32 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u32);
            }
        } else {
            const MAX_P1: f32 = u32::MAX as f32;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u32);
            }
        }
        None
    }

    #[inline]
    fn to_u64(&self) -> Option<u64> {
        if size_of::<f32>() > size_of::<u64>() {
            const MAX_P1: f32 = u64::MAX as f32 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u64);
            }
        } else {
            const MAX_P1: f32 = u64::MAX as f32;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u64);
            }
        }
        None
    }

    #[inline]
    fn to_u128(&self) -> Option<u128> {
        if size_of::<f32>() > size_of::<u128>() {
            const MAX_P1: f32 = u128::MAX as f32 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u128);
            }
        } else {
            const MAX_P1: f32 = u128::MAX as f32;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u128);
            }
        }
        None
    }

    #[inline]
    fn to_f32(&self) -> Option<f32> {
        Some(*self)
    }

    #[inline]
    fn to_f64(&self) -> Option<f64> {
        Some(*self as f64)
    }
}

impl ToPrimitive for f64 {
    #[inline]
    fn to_isize(&self) -> Option<isize> {
        if size_of::<f64>() > size_of::<isize>() {
            const MIN_M1: f64 = isize::MIN as f64 - 1.0;
            const MAX_P1: f64 = isize::MAX as f64 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as isize);
            }
        } else {
            const MIN: f64 = isize::MIN as f64;
            const MAX_P1: f64 = isize::MAX as f64;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as isize);
            }
        }
        None
    }

    #[inline]
    fn to_i8(&self) -> Option<i8> {
        if size_of::<f64>() > size_of::<i8>() {
            const MIN_M1: f64 = i8::MIN as f64 - 1.0;
            const MAX_P1: f64 = i8::MAX as f64 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as i8);
            }
        } else {
            const MIN: f64 = i8::MIN as f64;
            const MAX_P1: f64 = i8::MAX as f64;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as i8);
            }
        }
        None
    }

    #[inline]
    fn to_i16(&self) -> Option<i16> {
        if size_of::<f64>() > size_of::<i16>() {
            const MIN_M1: f64 = i16::MIN as f64 - 1.0;
            const MAX_P1: f64 = i16::MAX as f64 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as i16);
            }
        } else {
            const MIN: f64 = i16::MIN as f64;
            const MAX_P1: f64 = i16::MAX as f64;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as i16);
            }
        }
        None
    }

    #[inline]
    fn to_i32(&self) -> Option<i32> {
        if size_of::<f64>() > size_of::<i32>() {
            const MIN_M1: f64 = i32::MIN as f64 - 1.0;
            const MAX_P1: f64 = i32::MAX as f64 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as i32);
            }
        } else {
            const MIN: f64 = i32::MIN as f64;
            const MAX_P1: f64 = i32::MAX as f64;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as i32);
            }
        }
        None
    }

    #[inline]
    fn to_i64(&self) -> Option<i64> {
        if size_of::<f64>() > size_of::<i64>() {
            const MIN_M1: f64 = i64::MIN as f64 - 1.0;
            const MAX_P1: f64 = i64::MAX as f64 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as i64);
            }
        } else {
            const MIN: f64 = i64::MIN as f64;
            const MAX_P1: f64 = i64::MAX as f64;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as i64);
            }
        }
        None
    }

    #[inline]
    fn to_i128(&self) -> Option<i128> {
        if size_of::<f64>() > size_of::<i128>() {
            const MIN_M1: f64 = i128::MIN as f64 - 1.0;
            const MAX_P1: f64 = i128::MAX as f64 + 1.0;
            if *self > MIN_M1 && *self < MAX_P1 {
                return Some(*self as i128);
            }
        } else {
            const MIN: f64 = i128::MIN as f64;
            const MAX_P1: f64 = i128::MAX as f64;
            if *self >= MIN && *self < MAX_P1 {
                return Some(*self as i128);
            }
        }
        None
    }

    #[inline]
    fn to_usize(&self) -> Option<usize> {
        if size_of::<f64>() > size_of::<usize>() {
            const MAX_P1: f64 = usize::MAX as f64 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as usize);
            }
        } else {
            const MAX_P1: f64 = usize::MAX as f64;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as usize);
            }
        }
        None
    }

    #[inline]
    fn to_u8(&self) -> Option<u8> {
        if size_of::<f64>() > size_of::<u8>() {
            const MAX_P1: f64 = u8::MAX as f64 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u8);
            }
        } else {
            const MAX_P1: f64 = u8::MAX as f64;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u8);
            }
        }
        None
    }

    #[inline]
    fn to_u16(&self) -> Option<u16> {
        if size_of::<f64>() > size_of::<u16>() {
            const MAX_P1: f64 = u16::MAX as f64 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u16);
            }
        } else {
            const MAX_P1: f64 = u16::MAX as f64;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u16);
            }
        }
        None
    }

    #[inline]
    fn to_u32(&self) -> Option<u32> {
        if size_of::<f64>() > size_of::<u32>() {
            const MAX_P1: f64 = u32::MAX as f64 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u32);
            }
        } else {
            const MAX_P1: f64 = u32::MAX as f64;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u32);
            }
        }
        None
    }

    #[inline]
    fn to_u64(&self) -> Option<u64> {
        if size_of::<f64>() > size_of::<u64>() {
            const MAX_P1: f64 = u64::MAX as f64 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u64);
            }
        } else {
            const MAX_P1: f64 = u64::MAX as f64;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u64);
            }
        }
        None
    }

    #[inline]
    fn to_u128(&self) -> Option<u128> {
        if size_of::<f64>() > size_of::<u128>() {
            const MAX_P1: f64 = u128::MAX as f64 + 1.0;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u128);
            }
        } else {
            const MAX_P1: f64 = u128::MAX as f64;
            if *self > -1.0 && *self < MAX_P1 {
                return Some(*self as u128);
            }
        }
        None
    }

    #[inline]
    fn to_f32(&self) -> Option<f32> {
        Some(*self as f32)
    }

    #[inline]
    fn to_f64(&self) -> Option<f64> {
        Some(*self)
    }
}
