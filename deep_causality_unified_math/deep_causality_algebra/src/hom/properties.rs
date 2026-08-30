/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Hom;

/// A [`Hom`] that is **injective**: `f(a) == f(b)` implies `a == b`.
///
/// In categorical terms a monomorphism. The embeddings of the number tower are all injective and
/// none is surjective, which is exactly why they are not isomorphisms and could not be expressed
/// before: ℤ ↪ ℚ misses `1/2`, ℝ ↪ ℂ misses `i`.
///
/// An unverifiable promise, like the algebraic laws.
pub trait Injective: Hom {}

/// A [`Hom`] that is **surjective**: every element of the codomain is `f(x)` for some `x`.
///
/// In categorical terms an epimorphism — the projection side. `ℤ → ℤ/nℤ` is the archetype, and the
/// real part of a complex number, `ℂ → ℝ`, is surjective while being neither injective nor a
/// [`RingHom`](crate::RingHom): `re(i · i) = -1` but `re(i) · re(i) = 0`.
///
/// An unverifiable promise, like the algebraic laws.
pub trait Surjective: Hom {}

/// A [`Hom`] that is both [`Injective`] and [`Surjective`], hence invertible.
///
/// This is a **definition rather than a promise**, so unlike the other properties it is
/// blanket-implemented: a map that is injective and surjective *is* bijective, and asserting it
/// separately would let the two disagree. That is the same reasoning under which `Annihilating` is
/// stated on `Semiring` but is a theorem on `Ring`.
///
/// A bijective [`RingHom`](crate::RingHom) is a ring isomorphism — the map-level counterpart of the
/// pair-shaped traits in [`crate::iso`].
pub trait Bijective: Injective + Surjective {}

impl<T> Bijective for T where T: Injective + Surjective {}

/// A [`Bijective`] map that can produce its inverse — an **isomorphism**.
///
/// Named for what it is rather than `Invertible`, which this crate already uses for the
/// field-division marker `a · a⁻¹ = 1`. Different claim, different trait.
///
/// Bijectivity says an inverse *exists*; this says the map can hand it to you. Splitting them means
/// a map can be known bijective without the inverse being constructible, which is the usual
/// situation for an abstract existence argument.
///
/// The inverse's ends are the original's, swapped — which is the statement that only became
/// sayable once maps carried named ends.
pub trait Isomorphism: Bijective {
    /// The inverse map, running the other way.
    type Inverse: Hom<Domain = Self::Codomain, Codomain = Self::Domain>;

    /// Produce the inverse.
    fn inverse(&self) -> Self::Inverse;
}
