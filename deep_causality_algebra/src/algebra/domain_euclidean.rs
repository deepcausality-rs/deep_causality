/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::IntegralDomain;

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
/// The **integral domain** axioms come from the [`IntegralDomain`](crate::IntegralDomain)
/// supertrait, which states them on the rung they belong to:
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
pub trait EuclideanDomain: IntegralDomain {
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
    ///
    /// # Panics
    ///
    /// The divisor must be non-zero. That precondition is necessary but, on the fixed-width
    /// signed integers, not sufficient: `-1` satisfies it, yet `T::MIN.div_euclid(-1)` has the
    /// mathematical value `2^(N-1)`, which the type cannot hold. The primitive panics there in
    /// **every** build profile — signed division overflow is checked unconditionally rather than
    /// wrapped in release, because the hardware traps on it. ℤ is unbounded and has no such
    /// case, so this is a limit of the fixed-width representation and not of the Euclidean
    /// domain; it is documented rather than solved.
    fn div_euclid(&self, other: &Self) -> Self;

    /// Computes the remainder of Euclidean division.
    ///
    /// For `a.rem_euclid(b)`, returns `r` such that `a = b·q + r` with `0 ≤ r < |b|`.
    ///
    /// Unlike the `%` operator, the result is always non-negative.
    ///
    /// # Panics
    ///
    /// As for [`div_euclid`](Self::div_euclid), and for the same reason: a zero divisor has no
    /// remainder, and on the fixed-width signed integers `T::MIN.rem_euclid(-1)` panics in every
    /// build profile even though the answer, `0`, is representable — the quotient it is computed
    /// from is not.
    fn rem_euclid(&self, other: &Self) -> Self;

    /// Returns the canonical associate of `self`.
    ///
    /// A greatest common divisor is only defined **up to associates**: if `g` divides both
    /// arguments then so does `u·g` for any unit `u`, and both have equal claim to being "the"
    /// gcd. Over `ℤ` the units are `±1`, so `6` and `-6` are equally valid gcds of `48` and `18`.
    /// Fixing a representative is what lets [`gcd`](Self::gcd) return *a value* rather than an
    /// equivalence class.
    ///
    /// For `ℤ` the canonical associate is the absolute value, so `gcd` is non-negative wherever
    /// it is defined. For a polynomial ring `F[x]` it would be the monic representative.
    ///
    /// # Panics
    ///
    /// On the fixed-width signed integers this function is **partial**. It negates negative
    /// values, and `T::MIN` has no representable non-negative associate: `|i64::MIN|` is `2^63`,
    /// one past the top of the type. `T::MIN.normalize()` therefore panics in debug builds and
    /// wraps back to `T::MIN` — a *negative* value — in release, which is the one input where the
    /// non-negativity promise above cannot be kept. Over ℤ, which is unbounded, the promise holds
    /// everywhere; the gap is in the representation, not in the mathematics.
    ///
    /// [`checked_normalize`](Self::checked_normalize) is the same function with that input
    /// reported rather than hit. Callers for whom the non-negativity is load-bearing — reducing a
    /// fraction to a canonical form, say — should use it.
    fn normalize(&self) -> Self;

    /// Returns the canonical associate of `self`, or `None` when it is not representable.
    ///
    /// This is the total counterpart of [`normalize`](Self::normalize): every input either yields
    /// a canonical associate that satisfies the contract, or `None`. On the signed integers the
    /// sole `None` is `T::MIN`. A domain in which every element has a representable canonical
    /// associate returns `Some` for every input.
    fn checked_normalize(&self) -> Option<Self>
    where
        Self: Sized;

    /// Computes the greatest common divisor by the Euclidean algorithm.
    ///
    /// # Properties
    /// - `gcd(a, 0) = |a|`
    /// - `gcd(a, b) = gcd(b, a mod b)`
    /// - `gcd(a, b)` divides both `a` and `b`
    /// - every common divisor of `a` and `b` divides `gcd(a, b)`
    ///
    /// # Panics
    ///
    /// The result is normalized, so `gcd` is partial exactly where
    /// [`normalize`](Self::normalize) is: `i64::MIN.gcd(&0)` is `|i64::MIN|`, which the type
    /// cannot hold, so it panics in debug and wraps to a negative value in release. The `0 ≤
    /// gcd(a, b)` guarantee holds on every other input.
    ///
    /// The loop also calls [`rem_euclid`](Self::rem_euclid), so `gcd(T::MIN, -1)` panics on the
    /// first step, in every build profile, for the separate reason documented there.
    ///
    /// [`checked_gcd`](Self::checked_gcd) reports the unrepresentable-result case as `None`.
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

    /// Computes the greatest common divisor, returning `None` when the result is not
    /// representable.
    ///
    /// Same algorithm as [`gcd`](Self::gcd), closed off with
    /// [`checked_normalize`](Self::checked_normalize) instead of `normalize`. On the signed
    /// integers the `None` case is a gcd of `|T::MIN|`, which is where the algorithm terminates
    /// on `T::MIN` itself — `checked_gcd(T::MIN, 0)` and `checked_gcd(T::MIN, T::MIN)`. Whenever
    /// this returns `Some(g)`, `g` is the canonical associate and so satisfies `0 ≤ g`.
    ///
    /// `None` covers an unrepresentable *result*, not an unrepresentable *intermediate*: the
    /// `(T::MIN, -1)` case of [`rem_euclid`](Self::rem_euclid) still panics inside the loop,
    /// before any value can be returned.
    fn checked_gcd(&self, other: &Self) -> Option<Self>
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
        a.checked_normalize()
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
    /// in the formula above. It is partial wherever [`normalize`](Self::normalize) and
    /// [`gcd`](Self::gcd) are.
    ///
    /// The quotient is taken with [`div_euclid`](Self::div_euclid) rather than with the `/`
    /// operator. A domain is free to implement `Div` as something other than its Euclidean
    /// quotient — `Div` carries no law that ties the two together — so dividing through the
    /// operator would compute the least common multiple of a *different* division than the one
    /// this trait defines. `div_euclid` is the division the trait is about.
    fn lcm(&self, other: &Self) -> Self
    where
        Self: Sized + Clone,
    {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }
        let g = self.gcd(other);
        // `g` divides `self` exactly, so the quotient is exact and no precision is lost.
        (self.div_euclid(&g) * other.clone()).normalize()
    }
}

// -----------------------------------------------------------------------------
// Signed integer implementations
//
// ℤ is the motivating Euclidean domain, with φ(n) = |n|. The unsigned types are deliberately
// absent: they are not `CommutativeRing`, because ℕ has no additive inverses.
// -----------------------------------------------------------------------------

// Written out one width at a time rather than generated by a macro. `AGENTS.md` steers new library
// code away from macros, and the six bodies are not copies of each other in any case: the
// `EuclideanValue` associated type changes with the width (`i8 -> u8`, `i16 -> u16`, and so on),
// which a reader can check by eye. `normalize` is `abs`, which is partial at `T::MIN`;
// `checked_normalize` is `checked_abs`, which reports that input as `None`.

impl EuclideanDomain for i8 {
    type EuclideanValue = u8;

    #[inline]
    fn euclidean_fn(&self) -> Self::EuclideanValue {
        self.unsigned_abs()
    }

    #[inline]
    fn div_euclid(&self, other: &Self) -> Self {
        i8::div_euclid(*self, *other)
    }

    #[inline]
    fn rem_euclid(&self, other: &Self) -> Self {
        i8::rem_euclid(*self, *other)
    }

    #[inline]
    fn normalize(&self) -> Self {
        i8::abs(*self)
    }

    #[inline]
    fn checked_normalize(&self) -> Option<Self> {
        i8::checked_abs(*self)
    }
}

impl EuclideanDomain for i16 {
    type EuclideanValue = u16;

    #[inline]
    fn euclidean_fn(&self) -> Self::EuclideanValue {
        self.unsigned_abs()
    }

    #[inline]
    fn div_euclid(&self, other: &Self) -> Self {
        i16::div_euclid(*self, *other)
    }

    #[inline]
    fn rem_euclid(&self, other: &Self) -> Self {
        i16::rem_euclid(*self, *other)
    }

    #[inline]
    fn normalize(&self) -> Self {
        i16::abs(*self)
    }

    #[inline]
    fn checked_normalize(&self) -> Option<Self> {
        i16::checked_abs(*self)
    }
}

impl EuclideanDomain for i32 {
    type EuclideanValue = u32;

    #[inline]
    fn euclidean_fn(&self) -> Self::EuclideanValue {
        self.unsigned_abs()
    }

    #[inline]
    fn div_euclid(&self, other: &Self) -> Self {
        i32::div_euclid(*self, *other)
    }

    #[inline]
    fn rem_euclid(&self, other: &Self) -> Self {
        i32::rem_euclid(*self, *other)
    }

    #[inline]
    fn normalize(&self) -> Self {
        i32::abs(*self)
    }

    #[inline]
    fn checked_normalize(&self) -> Option<Self> {
        i32::checked_abs(*self)
    }
}

impl EuclideanDomain for i64 {
    type EuclideanValue = u64;

    #[inline]
    fn euclidean_fn(&self) -> Self::EuclideanValue {
        self.unsigned_abs()
    }

    #[inline]
    fn div_euclid(&self, other: &Self) -> Self {
        i64::div_euclid(*self, *other)
    }

    #[inline]
    fn rem_euclid(&self, other: &Self) -> Self {
        i64::rem_euclid(*self, *other)
    }

    #[inline]
    fn normalize(&self) -> Self {
        i64::abs(*self)
    }

    #[inline]
    fn checked_normalize(&self) -> Option<Self> {
        i64::checked_abs(*self)
    }
}

impl EuclideanDomain for i128 {
    type EuclideanValue = u128;

    #[inline]
    fn euclidean_fn(&self) -> Self::EuclideanValue {
        self.unsigned_abs()
    }

    #[inline]
    fn div_euclid(&self, other: &Self) -> Self {
        i128::div_euclid(*self, *other)
    }

    #[inline]
    fn rem_euclid(&self, other: &Self) -> Self {
        i128::rem_euclid(*self, *other)
    }

    #[inline]
    fn normalize(&self) -> Self {
        i128::abs(*self)
    }

    #[inline]
    fn checked_normalize(&self) -> Option<Self> {
        i128::checked_abs(*self)
    }
}

impl EuclideanDomain for isize {
    type EuclideanValue = usize;

    #[inline]
    fn euclidean_fn(&self) -> Self::EuclideanValue {
        self.unsigned_abs()
    }

    #[inline]
    fn div_euclid(&self, other: &Self) -> Self {
        isize::div_euclid(*self, *other)
    }

    #[inline]
    fn rem_euclid(&self, other: &Self) -> Self {
        isize::rem_euclid(*self, *other)
    }

    #[inline]
    fn normalize(&self) -> Self {
        isize::abs(*self)
    }

    #[inline]
    fn checked_normalize(&self) -> Option<Self> {
        isize::checked_abs(*self)
    }
}
