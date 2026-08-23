/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::CommutativeRing;
use core::ops::Div;

/// Represents a **Euclidean Domain**.
///
/// A Euclidean domain is an integral domain equipped with a Euclidean function that enables
/// division with remainder. It is the foundation of the Euclidean algorithm for greatest
/// common divisors, and it is the level of the tower at which exact integer arithmetic lives.
///
/// # Mathematical Definition
///
/// A commutative ring `R` is a Euclidean domain if there exists a function `φ: R \ {0} → ℕ`
/// (the Euclidean function) such that for any `a, b ∈ R` with `b ≠ 0` there exist `q, r ∈ R`
/// satisfying:
///
/// 1. `a = b·q + r`
/// 2. Either `r = 0` or `φ(r) < φ(b)`
///
/// For the integers, `φ(n) = |n|`.
///
/// # What implementing this promises
///
/// Beyond the division law above, this trait carries the **integral domain** axioms, which the
/// compiler cannot check:
///
/// - `1 ≠ 0`, and
/// - there are no zero divisors: `a·b = 0` implies `a = 0` or `b = 0`.
///
/// The absence of zero divisors is what licenses cancellation, and therefore what makes exact
/// elimination over the ring well defined. Do not implement this trait for a ring that has
/// them (`ℤ/6ℤ`, for instance, where `2·3 = 0`).
///
/// # Properties
/// - Every Euclidean domain is a Principal Ideal Domain (PID).
/// - Every Euclidean domain is a Unique Factorization Domain (UFD).
/// - The Euclidean algorithm terminates in finitely many steps.
///
/// # Examples
/// - Integers `ℤ` with `φ(n) = |n|`
/// - Gaussian integers `ℤ[i]` with `φ(a + bi) = a² + b²`
/// - Polynomial rings `F[x]` over a field with `φ(p) = deg(p)`
///
/// # Counter-examples
/// - The unsigned integers. `ℕ` has no additive inverses, so it is not an
///   [`AbelianGroup`](crate::AbelianGroup) and therefore not a
///   [`CommutativeRing`](crate::CommutativeRing) — it cannot reach this trait at all. A
///   Euclidean domain is a *ring* first, and `ℕ` is only a semiring.
/// - `ℤ/6ℤ`, which is a commutative ring but has zero divisors.
pub trait EuclideanDomain: CommutativeRing {
    /// The Euclidean function's value type — a measure of "size", ordered so that the
    /// remainder can be shown to strictly decrease.
    ///
    /// For the integers this is the corresponding unsigned type, so that `φ(MIN)` is
    /// representable; `i32::MIN.unsigned_abs()` is `2_147_483_648`, which `i32` cannot hold.
    type EuclideanValue: Ord;

    /// Computes the Euclidean function `φ`.
    ///
    /// For integers this returns the absolute value.
    fn euclidean_fn(&self) -> Self::EuclideanValue;

    /// Computes the quotient of Euclidean division.
    ///
    /// For `a.div_euclid(b)`, returns `q` such that `a = b·q + r` with `0 ≤ r < |b|`.
    fn div_euclid(&self, other: &Self) -> Self;

    /// Computes the remainder of Euclidean division.
    ///
    /// For `a.rem_euclid(b)`, returns `r` such that `a = b·q + r` with `0 ≤ r < |b|`.
    ///
    /// Unlike the `%` operator, the result is always non-negative.
    fn rem_euclid(&self, other: &Self) -> Self;

    /// Computes the greatest common divisor by the Euclidean algorithm.
    ///
    /// # Properties
    /// - `gcd(a, 0) = |a|`
    /// - `gcd(a, b) = gcd(b, a mod b)`
    /// - `gcd(a, b)` divides both `a` and `b`
    /// - every common divisor of `a` and `b` divides `gcd(a, b)`
    fn gcd(&self, other: &Self) -> Self
    where
        Self: Sized + Clone,
    {
        let mut a = self.clone();
        let mut b = other.clone();
        while !b.is_zero() {
            let r = a.rem_euclid(&b);
            a = b;
            b = r;
        }
        a
    }

    /// Computes the least common multiple, `lcm(a, b) = |a·b| / gcd(a, b)`.
    ///
    /// Returns zero when either argument is zero.
    fn lcm(&self, other: &Self) -> Self
    where
        Self: Sized + Clone + Div<Output = Self>,
    {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }
        let g = self.gcd(other);
        self.clone() * other.clone() / g
    }
}

// -----------------------------------------------------------------------------
// Signed integer implementations
//
// ℤ is the motivating Euclidean domain, with φ(n) = |n|. The unsigned types are deliberately
// absent: they are not `CommutativeRing`, because ℕ has no additive inverses.
// -----------------------------------------------------------------------------

macro_rules! impl_euclidean_domain_signed {
    ($($t:ty => $u:ty),* $(,)?) => {
        $(
            impl EuclideanDomain for $t {
                type EuclideanValue = $u;

                #[inline]
                fn euclidean_fn(&self) -> Self::EuclideanValue {
                    self.unsigned_abs()
                }

                #[inline]
                fn div_euclid(&self, other: &Self) -> Self {
                    <$t>::div_euclid(*self, *other)
                }

                #[inline]
                fn rem_euclid(&self, other: &Self) -> Self {
                    <$t>::rem_euclid(*self, *other)
                }
            }
        )*
    };
}

impl_euclidean_domain_signed!(
    i8 => u8,
    i16 => u16,
    i32 => u32,
    i64 => u64,
    i128 => u128,
    isize => usize,
);
