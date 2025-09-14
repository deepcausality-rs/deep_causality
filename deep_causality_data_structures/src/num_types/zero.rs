/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::ops::Add;

/// Defines an additive identity element for `Self`.
///
/// # Laws
///
/// ```text
/// a + 0 = a       ∀ a ∈ Self
/// 0 + a = a       ∀ a ∈ Self
/// ```
pub trait Zero: Sized + Add<Self, Output = Self> {
    /// Returns the additive identity element of `Self`, `0`.
    /// # Purity
    ///
    /// This function should return the same result at all times regardless of
    /// external mutable state.
    // This cannot be an associated constant, because of bignums.
    fn zero() -> Self;

    /// Sets `self` to the additive identity element of `Self`, `0`.
    fn set_zero(&mut self) {
        *self = Zero::zero();
    }

    /// Returns `true` if `self` is equal to the additive identity.
    fn is_zero(&self) -> bool;
}

/// Defines an associated constant representing the additive identity element
/// for `Self`.
pub trait ConstZero: Zero {
    /// The additive identity element of `Self`, `0`.
    const ZERO: Self;
}

impl Zero for usize {
    #[inline]
    fn zero() -> usize {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for usize {
    const ZERO: Self = 0;
}

impl Zero for u8 {
    #[inline]
    fn zero() -> u8 {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for u8 {
    const ZERO: Self = 0;
}

impl Zero for u16 {
    #[inline]
    fn zero() -> u16 {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for u16 {
    const ZERO: Self = 0;
}

impl Zero for u32 {
    #[inline]
    fn zero() -> u32 {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for u32 {
    const ZERO: Self = 0;
}

impl Zero for u64 {
    #[inline]
    fn zero() -> u64 {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for u64 {
    const ZERO: Self = 0;
}

impl Zero for u128 {
    #[inline]
    fn zero() -> u128 {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for u128 {
    const ZERO: Self = 0;
}

impl Zero for isize {
    #[inline]
    fn zero() -> isize {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for isize {
    const ZERO: Self = 0;
}

impl Zero for i8 {
    #[inline]
    fn zero() -> i8 {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for i8 {
    const ZERO: Self = 0;
}

impl Zero for i16 {
    #[inline]
    fn zero() -> i16 {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for i16 {
    const ZERO: Self = 0;
}

impl Zero for i32 {
    #[inline]
    fn zero() -> i32 {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for i32 {
    const ZERO: Self = 0;
}

impl Zero for i64 {
    #[inline]
    fn zero() -> i64 {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for i64 {
    const ZERO: Self = 0;
}

impl Zero for i128 {
    #[inline]
    fn zero() -> i128 {
        0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0
    }
}
impl ConstZero for i128 {
    const ZERO: Self = 0;
}

impl Zero for f32 {
    #[inline]
    fn zero() -> f32 {
        0.0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0.0
    }
}
impl ConstZero for f32 {
    const ZERO: Self = 0.0;
}

impl Zero for f64 {
    #[inline]
    fn zero() -> f64 {
        0.0
    }
    #[inline]
    fn is_zero(&self) -> bool {
        *self == 0.0
    }
}
impl ConstZero for f64 {
    const ZERO: Self = 0.0;
}
