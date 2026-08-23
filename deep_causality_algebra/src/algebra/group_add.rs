/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */
use crate::Zero;
use core::ops::{Add, Neg, Sub};

/// Represents an **Additive Group**.
///
/// An additive group is a `Group` where the binary operation is addition (`+`).
///
/// # Mathematical Definition
///
/// A set `G` is a group under addition if it satisfies:
/// 1.  **Closure:** `a + b` is in `G`. (Implicit in Rust).
/// 2.  **Associativity:** `(a + b) + c = a + (b + c)`. (Implied by `Add` trait).
/// 3.  **Identity Element:** There is an element `0` such that `a + 0 = a`.
///     (Provided by the `Zero` trait).
/// 4.  **Inverse Element:** For each `a`, there is an inverse `-a` such that
///     `a + (-a) = 0`. (Provided by the `Neg` trait.)
///
/// The `Clone` bound is included for practical purposes within the Rust type system.
///
/// # Why `Neg` and not `Sub`
///
/// The inverse axiom needs `-a`, and only `Neg` supplies it. `Sub` gives `a - a = 0`, which is a
/// weaker statement that a commutative *monoid* with a truncating difference also satisfies:
/// `3u64 - 3u64` is `0`, yet `u64` has no additive inverses at all.
///
/// Requiring only `Sub` therefore admitted ℕ. `u64` satisfied `AddGroup`, so
/// `fn inverse_of<T: AddGroup>(x: T) -> T { T::zero() - x }` type-checked at `u64` and returned
/// `18446744073709551615` in release builds. It also made the Lean claim
/// `algebra.add_group.neg_cancel` false for part of this trait's membership, since that theorem
/// is stated over Mathlib's `AddGroup`, which requires `neg`.
///
/// `Neg` is exactly the property separating ℤ from ℕ, and it is the same bound
/// [`AbelianGroup`](crate::AbelianGroup) uses for the same reason.
pub trait AddGroup:
    Add<Output = Self> + Sub<Output = Self> + Neg<Output = Self> + Zero + Clone
{
}

// Blanket Implementation for all types that impl Add, Sub, Neg, and have zero
impl<T> AddGroup for T where T: Add<Output = T> + Sub<Output = T> + Neg<Output = T> + Zero + Clone {}
