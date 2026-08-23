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

    /// Returns the canonical associate of `self`.
    ///
    /// A greatest common divisor is only defined **up to associates**: if `g` divides both
    /// arguments then so does `u·g` for any unit `u`, and both have equal claim to being "the"
    /// gcd. Over `ℤ` the units are `±1`, so `6` and `-6` are equally valid gcds of `48` and `18`.
    /// Fixing a representative is what lets [`gcd`](Self::gcd) return *a value* rather than an
    /// equivalence class.
    ///
    /// For `ℤ` the canonical associate is the absolute value, so `gcd` is always non-negative.
    /// For a polynomial ring `F[x]` it would be the monic representative.
    ///
    /// # Overflow
    ///
    /// For the signed integers this negates negative values, so `T::MIN` has no representable
    /// canonical associate and overflows — a panic in debug builds, wrapping in release. This is
    /// the same asymmetry that makes `T::MIN.abs()` overflow.
    fn normalize(&self) -> Self;

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
        // `a` holds the last non-zero remainder, or a seed when the loop ran fewer than two
        // steps — `gcd(a, 0)` exits immediately, and `gcd(-24, -12)` exits after one step. Those
        // seeds carry their own sign, so the result is normalized here rather than assumed
        // non-negative: without this, `gcd(-7, 0)` would be `-7`.
        a.normalize()
    }

    /// Computes the least common multiple, `lcm(a, b) = |a·b| / gcd(a, b)`.
    ///
    /// Returns zero when either argument is zero.
    ///
    /// The division is performed **before** the multiplication. Forming `a·b` first overflows
    /// whenever the product exceeds the width even though the result would fit: `lcm(2⁴⁰, 2⁴⁰)`
    /// is `2⁴⁰`, but `2⁴⁰ · 2⁴⁰` is `2⁸⁰` and does not fit in an `i64`. Dividing one operand by
    /// the gcd first keeps the intermediate no larger than the answer.
    ///
    /// The result is normalized, so it is non-negative for the integers — matching the `|a·b|`
    /// in the formula above.
    fn lcm(&self, other: &Self) -> Self
    where
        Self: Sized + Clone + Div<Output = Self>,
    {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }
        let g = self.gcd(other);
        // `g` divides `self` exactly, so `self / g` is exact and no precision is lost.
        ((self.clone() / g) * other.clone()).normalize()
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

                #[inline]
                fn normalize(&self) -> Self {
                    <$t>::abs(*self)
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
