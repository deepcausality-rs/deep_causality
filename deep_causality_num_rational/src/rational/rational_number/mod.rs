/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{EuclideanDomain, SignedInt};
use core::ops::Div;

mod algebra;
mod arithmetic;
mod display;
mod identity;
mod ord;

/// The scalar a [`Rational`] is built over.
///
/// Mathematically, the field of fractions exists over any integral domain. This crate builds it
/// over the **ordered, fixed-width signed integers**, which is narrower, and deliberately so:
///
/// - [`EuclideanDomain`] — reducing a fraction to lowest terms needs a `gcd`, and the Euclidean
///   algorithm is what provides one.
/// - [`SignedInt`] — canonical form needs three things that a bare `PartialOrd` cannot supply.
///   It needs a **total** order, because [`Ord`] for a `Rational` has no way to answer
///   "incomparable"; it needs a **sign**, because invariant 1 below moves that sign into the
///   numerator; and it needs to know where the range **ends**, because `T::MIN` is the one value
///   whose negation does not fit, and the difference between an honest type and a wrong one is
///   whether that case is detected (`checked_neg`) or silently wrapped.
/// - `Div` — to divide numerator and denominator through by their gcd.
///
/// An earlier form of this bound asked only for `EuclideanDomain + Neg + PartialOrd`. That was
/// too weak to be honest: a Euclidean domain with incomparable elements — the Gaussian integers
/// `ℤ[i]`, say — would have satisfied it, and for such a type "the denominator is positive" is
/// not an invariant but a category error, while `Ord::cmp` would have had to invent an answer.
///
/// Every signed integer width satisfies the bound: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`.
/// The unsigned types do not, and cannot: ℕ is not a ring, so it is not a `EuclideanDomain`, and
/// a field of fractions over it would have nothing to negate.
pub trait RationalScalar: EuclideanDomain + SignedInt + Div<Output = Self> {}

impl<T> RationalScalar for T where T: EuclideanDomain + SignedInt + Div<Output = T> {}

/// A rational number `n/d` — an element of the field of fractions of `T`.
///
/// `Rational<T>` is **exact**. Unlike a floating-point value, `1/3` is represented as the pair
/// `(1, 3)` and never as an approximation of it, so `(1/3) * 3` is exactly `1`.
///
/// # Invariants
///
/// Every `Rational` is kept in canonical form, which is why the fields are private:
///
/// 1. The denominator is strictly positive. A sign lives in the numerator, never the denominator.
/// 2. The numerator and denominator are coprime — `gcd(n, d) == 1`.
/// 3. Zero is exactly `0/1`.
/// 4. The numerator is never `T::MIN`.
///
/// Canonical form makes equality structural: two `Rational`s are equal exactly when their
/// components match, so `PartialEq` needs no cross-multiplication.
///
/// Invariant 4 is the one that is about the machine rather than about ℚ. `T::MIN` has no
/// representable negation — `-i64::MIN` is `2⁶³`, one past the top of the range — so a numerator
/// holding it would make [`Neg`](core::ops::Neg) partial, and with it subtraction, and with it
/// the `AbelianGroup` marker this type claims. Excluding it at construction costs one value per
/// width and buys a negation that cannot fail. The excluded set is symmetric, so the field laws
/// are untouched: `Rational<i64>` represents every `p/q` with `|p| ≤ i64::MAX` and
/// `0 < q ≤ i64::MAX`, and that set is closed under negation and reciprocal.
///
/// # Where this sits in the tower
///
/// ℚ is a [`Field`](deep_causality_algebra::Field): every non-zero rational has a multiplicative
/// inverse, `(n/d)⁻¹ = d/n`. It is **not** a [`Real`](deep_causality_algebra::Real), and that is
/// not an omission. `Real` is the analytic axis — `sqrt`, `exp`, `ln`, `sin` — and ℚ is not
/// closed under any of them: `sqrt(2)` is irrational, which is the oldest theorem about this
/// gap. ℚ is arithmetically complete and analytically empty.
///
/// This mirrors ℤ, which reaches [`CommutativeRing`](deep_causality_algebra::CommutativeRing) and
/// stops short of `Field` for the opposite reason — ℤ has no inverses. Passing from ℤ to ℚ buys
/// division and gives up nothing; passing from ℚ to ℝ buys limits and gives up exactness.
///
/// # Overflow
///
/// `T` is fixed-width, so not every exact result of an operation on representable values is
/// itself representable. That much is inherent to fixed-width rational arithmetic. What is not
/// inherent is *which* operations can fail, and this section is precise about it rather than
/// blanket.
///
/// **Total — cannot overflow, on any input:**
///
/// - **Construction.** [`try_new`](Self::try_new) returns `None` for every pair with no canonical
///   form and a correct value for every pair that has one. It reduces by the gcd *before* moving
///   a sign out of the denominator, so `try_new(i64::MIN, -2)` is `2⁶²` rather than an overflow
///   on an intermediate `-i64::MIN` that was never needed.
/// - **Comparison.** [`Ord`] compares by continued fraction — floor quotients, then the
///   remainders — and multiplies nothing. `Rational::new(i64::MAX, 1)` and `Rational::new(1, 2)`
///   compare correctly.
/// - **Negation, and therefore subtraction's use of it, and [`recip`](Self::recip).** Guaranteed
///   by invariant 4.
///
/// **Partial — can overflow, and here is exactly when:**
///
/// - **Addition and subtraction.** Each operand is split into an integer part and a proper
///   fraction, `a/b = q + r/b`, and the two are carried separately. The consequence is that a sum
///   whose *integer* part is large no longer overflows on the numerator: `MAX/2 + MAX/2` is
///   exactly `MAX`, where forming `MAX + MAX` would not fit. It overflows when the least common
///   denominator `lcm(b, d)` does not fit, when `q₁ + q₂` does not fit, or when the fractional
///   numerators `r₁·(d/g) + r₂·(b/g)` — bounded by `2·lcm(b, d)` — do not. Two coprime
///   denominators near `√T::MAX` will therefore still overflow, and no arrangement of the
///   arithmetic can prevent it: the answer itself is not representable.
/// - **Multiplication and division.** Each numerator is cross-cancelled against the *other*
///   denominator before either product is formed, so `2/3 · 3/2` never builds `6/6`. What is left
///   after cancelling must fit: the product of two coprime numerators, and of two coprime
///   denominators.
/// - **Anything whose exact answer needs a numerator of `T::MIN`,** which invariant 4 excludes.
///
/// When an operation does overflow, the behaviour is `T`'s — a panic in debug builds, wrapping in
/// release — except where the type can see the failure itself, which is when it panics with a
/// message naming the cause. Choose `T` with headroom (`i128` where the denominators are
/// unpredictable), and prefer accumulating over a common denominator where the problem admits
/// one.
///
/// # Examples
///
/// ```
/// use deep_causality_num_rational::Rational;
///
/// let third = Rational::new(1_i64, 3);
/// let sum = third + third + third;
/// assert_eq!(sum, Rational::from_integer(1)); // exact, unlike 0.1 + 0.2 in binary floats
///
/// // Fractions are reduced on construction.
/// assert_eq!(Rational::new(6_i64, 8), Rational::new(3, 4));
///
/// // A sign never survives in the denominator.
/// let r = Rational::new(1_i64, -2);
/// assert_eq!(*r.numer(), -1);
/// assert_eq!(*r.denom(), 2);
///
/// // Reduction happens before the sign moves, so this is +2⁶² and not an overflow.
/// assert_eq!(Rational::try_new(i64::MIN, -2), Rational::try_new(4_611_686_018_427_387_904, 1));
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Rational<T: RationalScalar> {
    /// The numerator. Carries the sign, and is never `T::MIN`.
    num: T,
    /// The denominator. Always strictly positive, never zero.
    den: T,
}

impl<T: RationalScalar> Rational<T> {
    /// Constructs `num/den`, reduced to canonical form.
    ///
    /// # Panics
    ///
    /// Panics if `den` is zero. Division by zero has no value in ℚ, so there is nothing this
    /// could return. This matches the behaviour of `/` on the integers themselves.
    ///
    /// Panics if the exact value has no canonical form in `T` — either because it needs a
    /// numerator or denominator of magnitude `|T::MIN|`, which does not fit, or because it needs
    /// a numerator of exactly `T::MIN`, which invariant 4 excludes. `try_new(i64::MIN, -1)` is
    /// `2⁶³` and cannot be represented at all; `try_new(i64::MIN, 1)` could be, but is refused so
    /// that negation stays total.
    ///
    /// Use [`try_new`](Self::try_new) wherever either case is possible.
    #[inline]
    pub fn new(num: T, den: T) -> Self {
        if den.is_zero() {
            panic!("Rational::new called with a zero denominator");
        }
        match Self::try_reduce(num, den) {
            Some(r) => r,
            None => panic!(
                "Rational::new called with a value that has no canonical form in T; \
                 the signed minimum has no representable negation"
            ),
        }
    }

    /// Constructs `num/den`, reduced to canonical form, or `None` if it has none.
    ///
    /// The total counterpart of [`new`](Self::new), and total in the strong sense: it returns a
    /// correct value or nothing, never a wrong one, for every pair of inputs including the
    /// extremes of the range.
    ///
    /// `None` means one of:
    ///
    /// - `den` is zero. Division by zero has no value in ℚ.
    /// - The reduced value needs a component of magnitude `|T::MIN|`, which `T` cannot hold —
    ///   `try_new(i64::MIN, -1)` is `2⁶³`, and `try_new(3, i64::MIN)` needs a denominator of
    ///   `2⁶³`.
    /// - The reduced numerator is exactly `T::MIN`, which invariant 4 excludes so that negation
    ///   is total.
    ///
    /// Everything else has a value, *after* reduction: `try_new(i64::MIN, -2)` is `2⁶²`, and
    /// `try_new(4, i64::MIN)` is `-1/2⁶¹`.
    #[inline]
    pub fn try_new(num: T, den: T) -> Option<Self> {
        if den.is_zero() {
            return None;
        }
        Self::try_reduce(num, den)
    }

    /// Constructs the rational `n/1` from an integer.
    ///
    /// This is the canonical embedding ℤ ↪ ℚ, and it is injective: distinct integers give
    /// distinct rationals.
    ///
    /// # Panics
    ///
    /// Panics if `n` is `T::MIN`, which invariant 4 excludes. The embedding is therefore of
    /// `[T::MIN + 1, T::MAX]` rather than of all of `T` — the fixed-width range is asymmetric,
    /// and this is where that asymmetry is paid for. Use
    /// [`try_from_integer`](Self::try_from_integer) where `n` may be the minimum.
    #[inline]
    pub fn from_integer(n: T) -> Self {
        match Self::try_from_integer(n) {
            Some(r) => r,
            None => panic!(
                "Rational::from_integer called with T::MIN, which has no representable negation"
            ),
        }
    }

    /// Constructs the rational `n/1`, or `None` if `n` is `T::MIN`.
    ///
    /// The total counterpart of [`from_integer`](Self::from_integer).
    #[inline]
    pub fn try_from_integer(n: T) -> Option<Self> {
        if n == T::MIN {
            return None;
        }
        Some(Self {
            num: n,
            den: T::one(),
        })
    }

    /// Returns the numerator, which carries the sign.
    #[inline]
    pub fn numer(&self) -> &T {
        &self.num
    }

    /// Returns the denominator, which is always strictly positive.
    #[inline]
    pub fn denom(&self) -> &T {
        &self.den
    }

    /// Returns `true` if the denominator is one, so the value is an integer.
    #[inline]
    pub fn is_integer(&self) -> bool {
        self.den == T::one()
    }

    /// Returns the multiplicative inverse `d/n`, or `None` for zero.
    ///
    /// Zero is the one element of a field with no inverse, so `None` is the honest answer rather
    /// than a panic. For every other value this is total: swapping the components of a canonical
    /// pair gives a canonical pair, once the sign is moved back to the numerator.
    #[inline]
    pub fn checked_recip(&self) -> Option<Self> {
        if self.num.is_zero() {
            return None;
        }
        // The components are already coprime, so the swap needs only the sign fixed up. Both
        // negations below are safe: the denominator is positive, and the numerator is never
        // `T::MIN` (invariant 4).
        if self.num < T::zero() {
            Some(Self {
                num: -self.den,
                den: -self.num,
            })
        } else {
            Some(Self {
                num: self.den,
                den: self.num,
            })
        }
    }

    /// Returns the multiplicative inverse `d/n`.
    ///
    /// # Panics
    ///
    /// Panics if the value is zero.
    #[inline]
    pub fn recip(&self) -> Self {
        match self.checked_recip() {
            Some(r) => r,
            None => panic!("Rational::recip called on zero"),
        }
    }

    /// Reduces `num/den` to canonical form, panicking if it has none.
    ///
    /// The caller guarantees `den != 0`. Used by the arithmetic operators, whose signatures
    /// return `Self` and so have nowhere to put a `None`.
    #[inline]
    pub(crate) fn reduce(num: T, den: T) -> Self {
        match Self::try_reduce(num, den) {
            Some(r) => r,
            None => panic!(
                "Rational: the result has no canonical form in T; \
                 the signed minimum has no representable negation"
            ),
        }
    }

    /// Reduces `num/den` to canonical form, or `None` if it has none.
    ///
    /// The caller guarantees `den != 0`.
    ///
    /// The order of the two steps is what makes this correct at the edge of the range. Moving the
    /// sign out of the denominator first, as an earlier version did, negates a numerator that may
    /// still be `T::MIN` — so `(i64::MIN)/(-2)` overflowed on `-i64::MIN` and returned `-2⁶²`,
    /// the *negative* of the right answer, in release. Dividing by the gcd first shrinks the
    /// magnitude to something the negation can hold: `(i64::MIN)/(-2)` becomes `(-2⁶²)/(-1)`
    /// becomes `2⁶²`. Whatever is still out of range after that is rejected rather than wrapped.
    #[inline]
    pub(crate) fn try_reduce(num: T, den: T) -> Option<Self> {
        debug_assert!(!den.is_zero(), "try_reduce called with a zero denominator");

        // Zero is `0/1` (invariant 3). Taking it first is also what keeps `gcd(0, T::MIN)` —
        // whose value is `|T::MIN|`, and therefore not representable — from ever being formed.
        if num.is_zero() {
            return Some(Self {
                num: T::zero(),
                den: T::one(),
            });
        }

        // `num == den` is the only other pair whose gcd is unrepresentable, and only when both
        // are `T::MIN`. Its value is exactly one, so answer it without a gcd at all.
        if num == den {
            return Some(Self {
                num: T::one(),
                den: T::one(),
            });
        }

        let g = Self::gcd_in_range(num, den);
        let n = num / g;
        let d = den / g;

        // Reduce first, negate second — see the doc comment above.
        let (n, d) = if d < T::zero() {
            (n.checked_neg()?, d.checked_neg()?)
        } else {
            (n, d)
        };

        // Invariant 4. `checked_neg` never yields `T::MIN`, so this only ever fires on the
        // branch that did not negate, where `num` was already the minimum.
        if n == T::MIN {
            return None;
        }

        Some(Self { num: n, den: d })
    }

    /// The gcd of `num` and `den`, computed so that it cannot overflow.
    ///
    /// The caller guarantees both are non-zero and that they are not both `T::MIN`.
    ///
    /// [`EuclideanDomain::gcd`] normalizes its result, and that normalization overflows exactly
    /// when the result is `T::MIN` — which needs `|T::MIN|` to divide both arguments, so it needs
    /// both arguments to *be* `T::MIN`. Replacing a `T::MIN` argument by `T::MIN + |other|`
    /// leaves the gcd unchanged, because adding a multiple of the other argument is a step of the
    /// Euclidean algorithm, and moves it off the boundary. The addition itself cannot overflow:
    /// `|other| ≤ T::MAX`, so the sum stays negative.
    #[inline]
    fn gcd_in_range(num: T, den: T) -> T {
        let (a, b) = if num == T::MIN {
            (num + den.abs(), den)
        } else if den == T::MIN {
            (num, den + num.abs())
        } else {
            (num, den)
        };
        a.gcd(&b)
    }
}
