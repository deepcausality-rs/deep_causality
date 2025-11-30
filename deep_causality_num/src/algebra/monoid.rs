/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{One, Zero};
use core::ops::{Add, AddAssign, Mul, MulAssign};

/// An Additive Monoid is a set equipped with an associative binary operation (+)
/// and an identity element (0).
///
/// Laws:
/// 1. Associativity: (a + b) + c = a + (b + c)
/// 2. Identity: a + 0 = a = 0 + a
pub trait AddMonoid: Add<Output = Self> + AddAssign + Zero + Clone {}

// Blanket Implementation for all types that implement Add, AddAssign, and Zero
impl<T> AddMonoid for T where T: Add<Output = Self> + AddAssign + Zero + Clone {}

/// A Multiplicative Monoid is a set equipped with an associative binary operation (*)
/// and an identity element (1).
///
/// Laws:
/// 1. Associativity: (a * b) * c = a * (b * c)
/// 2. Identity: a * 1 = a = 1 * a
pub trait MulMonoid: Mul<Output = Self> + MulAssign + One + Clone {}

// Blanket Implementation for all types that implement Mul, MulAssign, and One
impl<T> MulMonoid for T where T: Mul<Output = Self> + MulAssign + One + Clone {}
