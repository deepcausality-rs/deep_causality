/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::{One, Zero};
use core::ops::{Add, AddAssign, Mul, MulAssign};

/// Represents an **Additive Monoid**.
///
/// A monoid is an algebraic structure with a single associative binary
/// operation and an identity element. An additive monoid is one where the
/// operation is addition (`+`).
///
/// # Mathematical Definition
///
/// A set `S` with a binary operation `+` is an additive monoid if it satisfies:
/// 1.  **Closure:** `a + b` is in `S`. (Implicit in Rust).
/// 2.  **Associativity:** `(a + b) + c = a + (b + c)` for all `a, b, c` in `S`.
///     (A property the implementor must uphold).
/// 3.  **Identity Element:** There exists an element `0` in `S` such that
///     `a + 0 = 0 + a = a` for all `a` in `S`. (Provided by the `Zero` trait).
///
/// The `Clone` and `AddAssign` bounds are included for practical purposes.
pub trait AddMonoid: Add<Output = Self> + AddAssign + Zero + Clone {}

// Blanket Implementation for all types that implement Add, AddAssign, and Zero
impl<T> AddMonoid for T where T: Add<Output = Self> + AddAssign + Zero + Clone {}

/// Represents a **Multiplicative Monoid**.
///
/// A monoid is an algebraic structure with a single associative binary
/// operation and an identity element. A multiplicative monoid is one where the
/// operation is multiplication (`*`).
///
/// # Mathematical Definition
///
/// A set `S` with a binary operation `*` is a multiplicative monoid if it satisfies:
/// 1.  **Closure:** `a * b` is in `S`. (Implicit in Rust).
/// 2.  **Associativity:** `(a * b) * c = a * (b * c)` for all `a, b, c` in `S`.
///     (A property the implementor must uphold).
/// 3.  **Identity Element:** There exists an element `1` in `S` such that
///     `a * 1 = 1 * a = a` for all `a` in `S`. (Provided by the `One` trait).
///
/// The `Clone` and `MulAssign` bounds are included for practical purposes.
pub trait MulMonoid: Mul<Output = Self> + MulAssign + One + Clone {}

// Blanket Implementation for all types that implement Mul, MulAssign, and One
impl<T> MulMonoid for T where T: Mul<Output = Self> + MulAssign + One + Clone {}
