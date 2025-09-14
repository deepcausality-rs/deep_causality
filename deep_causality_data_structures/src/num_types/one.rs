/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use std::ops::Mul;

/// Defines a multiplicative identity element for `Self`.
///
/// # Laws
///
/// ```text
/// a * 1 = a       ∀ a ∈ Self
/// 1 * a = a       ∀ a ∈ Self
/// ```
pub trait One: Sized + Mul<Self, Output = Self> {
    /// Returns the multiplicative identity element of `Self`, `1`.
    ///
    /// # Purity
    ///
    /// This function should return the same result at all times regardless of
    /// external mutable state.
    // This cannot be an associated constant, because of bignums.
    fn one() -> Self;

    /// Sets `self` to the multiplicative identity element of `Self`, `1`.
    fn set_one(&mut self) {
        *self = One::one();
    }

    /// Returns `true` if `self` is equal to the multiplicative identity.
    ///
    /// For performance reasons, it's best to implement this manually.
    /// After a semver bump, this method will be required, and the
    /// `where Self: PartialEq` bound will be removed.
    #[inline]
    fn is_one(&self) -> bool
    where
        Self: PartialEq,
    {
        *self == Self::one()
    }
}

/// Defines an associated constant representing the multiplicative identity
/// element for `Self`.
pub trait ConstOne: One {
    /// The multiplicative identity element of `Self`, `1`.
    const ONE: Self;
}

impl One for usize {
    #[inline]
    fn one() -> usize {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for usize {
    const ONE: Self = 1;
}

impl One for u8 {
    #[inline]
    fn one() -> u8 {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for u8 {
    const ONE: Self = 1;
}

impl One for u16 {
    #[inline]
    fn one() -> u16 {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for u16 {
    const ONE: Self = 1;
}

impl One for u32 {
    #[inline]
    fn one() -> u32 {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for u32 {
    const ONE: Self = 1;
}

impl One for u64 {
    #[inline]
    fn one() -> u64 {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for u64 {
    const ONE: Self = 1;
}

impl One for u128 {
    #[inline]
    fn one() -> u128 {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for u128 {
    const ONE: Self = 1;
}

impl One for isize {
    #[inline]
    fn one() -> isize {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for isize {
    const ONE: Self = 1;
}

impl One for i8 {
    #[inline]
    fn one() -> i8 {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for i8 {
    const ONE: Self = 1;
}

impl One for i16 {
    #[inline]
    fn one() -> i16 {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for i16 {
    const ONE: Self = 1;
}

impl One for i32 {
    #[inline]
    fn one() -> i32 {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for i32 {
    const ONE: Self = 1;
}

impl One for i64 {
    #[inline]
    fn one() -> i64 {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for i64 {
    const ONE: Self = 1;
}

impl One for i128 {
    #[inline]
    fn one() -> i128 {
        1
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1
    }
}
impl ConstOne for i128 {
    const ONE: Self = 1;
}

impl One for f32 {
    #[inline]
    fn one() -> f32 {
        1.0
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1.0
    }
}
impl ConstOne for f32 {
    const ONE: Self = 1.0;
}

impl One for f64 {
    #[inline]
    fn one() -> f64 {
        1.0
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == 1.0
    }
}
impl ConstOne for f64 {
    const ONE: Self = 1.0;
}
