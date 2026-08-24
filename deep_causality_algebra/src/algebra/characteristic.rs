/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Field, Float106};

/// The **characteristic** of a ring: the least `n > 0` with `n · 1 = 0`, or `0` when no such `n`
/// exists.
///
/// # Why the tower needs this
///
/// [`Field`](crate::Field) is blanket-implemented. A type becomes a field the moment it satisfies
/// `CommutativeRing + InvMonoid + Div + DivAssign`, with no per-type opt-in — so admitting a finite
/// field widens every `T: Field` bound in the workspace at once, and nothing marks which of them can
/// take it.
///
/// What such a bound usually assumes, without saying so, is that `T::one() + T::one()` is a value
/// worth dividing by. Over ℚ, ℝ and ℂ it is two. Over 𝔽₂ it is zero.
///
/// # Characteristic, not size
///
/// The obvious guard is a finite-versus-infinite split, and it is the wrong one. Finiteness neither
/// implies nor follows from the property the code depends on:
///
/// | field | finite | characteristic | halving |
/// |---|---|---|---|
/// | ℚ, ℝ, ℂ | no | 0 | works |
/// | 𝔽₂ | yes | 2 | `1 + 1 = 0` — fails |
/// | 𝔽₃ | yes | 3 | `1 + 1 = 2 ≠ 0` — works |
/// | 𝔽₄ | yes | 2 | fails |
/// | 𝔽ₚ(x) | **no** | p | fails |
///
/// The property is `n · 1 ≠ 0` for every `n > 0` — characteristic zero — which is what
/// [`DivisibleByIntegers`] states.
///
/// # The value
///
/// The characteristic of a ring with unity is either `0` or a prime. `0` is the conventional value
/// for "no such `n`" and is the value the classical scalars report.
pub trait Characteristic {
    /// The characteristic: `0` when no positive multiple of `1` is zero, otherwise the least such
    /// multiple.
    const CHARACTERISTIC: u32;
}

/// A [`Field`](crate::Field) in which dividing by a non-zero integer is always defined.
///
/// This is the bound for any operation that divides by an integer literal. Halving is the case that
/// occurs in this workspace — `T::one() / (T::one() + T::one())` — but the promise is general: for
/// every `n > 0`, the element `n · 1` is non-zero and therefore invertible, so `x / n` exists for
/// every `x`.
///
/// # The textbook name
///
/// This is **characteristic zero**, stated as what it lets a caller do rather than as the property
/// it is. A field has characteristic zero when no positive integer multiple of `1` is zero, which is
/// exactly the condition above; [`Characteristic::CHARACTERISTIC`] reports it as the number `0`.
///
/// Equivalently, and perhaps more usefully: the canonical ring map `ℤ → F` is injective, so such a
/// field contains a copy of ℤ, and hence of ℚ. `IntToRational` in `deep_causality_num_rational` is
/// that embedding made explicit for ℚ.
///
/// The name is the consequence rather than the cause because the consequence is what a reader at a
/// bound site needs: `fn halve<T: DivisibleByIntegers>` says why the bound is there.
///
/// # What implementing this promises
///
/// That `n · 1 ≠ 0` for all `n > 0`, which the compiler cannot check. Like every law in this tower
/// it is a deliberate per-type assertion rather than something granted by a blanket, so that a
/// future field does not inherit the promise by satisfying a structural bound.
///
/// # Not a claim about ℤ
///
/// The supertrait is [`Field`](crate::Field), and it is load-bearing. ℤ has characteristic zero and
/// is not divisible by integers — `3 / 2` has no integer value — so the name would be false there.
/// It cannot be stated there: ℤ is not a field, and this trait requires one.
///
/// # What the bound does check
///
/// Which scalars may reach a body that halves. Over ℝ this is ordinary arithmetic:
///
/// ```
/// use deep_causality_algebra::DivisibleByIntegers;
/// use deep_causality_num::One;
///
/// fn halve<T: DivisibleByIntegers>(x: T) -> T {
///     x / (T::one() + T::one())
/// }
///
/// assert_eq!(halve(4.0_f64), 2.0);
/// ```
///
/// Over 𝔽₂ the same call is refused, because `1 + 1` is `0` there and the division would be by
/// zero. A `Field` bound would have admitted it — `Field` is blanket-implemented, so `Gf2` reaches
/// it the moment it is a `CommutativeRing` with division, without anyone opting in:
///
/// ```compile_fail,E0277
/// use deep_causality_algebra::DivisibleByIntegers;
/// use deep_causality_num::{Gf2, One};
///
/// fn halve<T: DivisibleByIntegers>(x: T) -> T {
///     x / (T::one() + T::one())
/// }
///
/// // `Gf2: DivisibleByIntegers` is not satisfied.
/// let _ = halve(Gf2::ONE);
/// ```
///
/// # Disjoint from [`FiniteField`], and not a partition
///
/// No type is both: every finite field has prime characteristic `p`, and `p · 1 = 0` there, so
/// dividing by `p` is undefined. The compiler does not enforce that — this MSRV has no negative
/// impls — so it is a discipline at the impl sites, stated at both.
///
/// The two do **not** partition the fields. `𝔽ₚ(x)`, the field of rational functions over `𝔽ₚ`, is
/// infinite and has characteristic `p`: it is neither finite nor divisible by every integer, since
/// dividing by `p` fails there too. Nothing in this workspace models it, but the traits are named
/// for what they mean rather than for the two cases that happen to be present, so that a third case
/// has somewhere to go.
pub trait DivisibleByIntegers: Field + Characteristic {}

/// A [`Field`](crate::Field) with finitely many elements.
///
/// # Order against characteristic
///
/// Every finite field has order `q = pᵏ` for a prime `p` and `k ≥ 1`, where `p` is its
/// characteristic. The two are different questions and 𝔽₄ is the case that shows it: order 4,
/// characteristic 2. Code that reduces modulo the characteristic needs
/// [`Characteristic::CHARACTERISTIC`]; code that enumerates the field or computes a Frobenius power
/// needs [`ORDER`](Self::ORDER). Exposing only one would make the other unreachable.
///
/// # Disjoint from [`DivisibleByIntegers`], and not a partition
///
/// See that trait. A finite field's characteristic is prime and therefore non-zero, and a field may
/// be in neither — `𝔽ₚ(x)` is infinite of characteristic `p`.
pub trait FiniteField: Field + Characteristic {
    /// The number of elements, `q = pᵏ`.
    const ORDER: u64;
}

// ℝ, as modelled by the floats. No positive multiple of 1.0 is 0.0 — the representation saturates
// to infinity rather than wrapping to zero, so the promise holds for every `n` the type can reach.
impl Characteristic for f32 {
    const CHARACTERISTIC: u32 = 0;
}
impl Characteristic for f64 {
    const CHARACTERISTIC: u32 = 0;
}
impl Characteristic for Float106 {
    const CHARACTERISTIC: u32 = 0;
}

impl DivisibleByIntegers for f32 {}
impl DivisibleByIntegers for f64 {}
impl DivisibleByIntegers for Float106 {}

// ℤ. Not a field, so it reaches neither refinement — but characteristic is a property of any ring
// with unity, and ℤ has characteristic zero. Stated because the trait means what it says.
impl Characteristic for i8 {
    const CHARACTERISTIC: u32 = 0;
}
impl Characteristic for i16 {
    const CHARACTERISTIC: u32 = 0;
}
impl Characteristic for i32 {
    const CHARACTERISTIC: u32 = 0;
}
impl Characteristic for i64 {
    const CHARACTERISTIC: u32 = 0;
}
impl Characteristic for i128 {
    const CHARACTERISTIC: u32 = 0;
}
impl Characteristic for isize {
    const CHARACTERISTIC: u32 = 0;
}

// 𝔽₂. `1 + 1 = 0` and there is no smaller positive multiple, so the characteristic is 2, and the
// order is 2 because the field is `{0, 1}`. Here `q = p¹`, so the two numbers coincide; 𝔽₄ is where
// they part.
//
// This type is deliberately NOT `DivisibleByIntegers`.
impl Characteristic for deep_causality_num::Gf2 {
    const CHARACTERISTIC: u32 = 2;
}

impl FiniteField for deep_causality_num::Gf2 {
    const ORDER: u64 = 2;
}
