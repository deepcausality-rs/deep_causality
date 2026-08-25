/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! Re-roots the witness-based isomorphisms on [`Hom`], so the crate has one notion of map.
//!
//! [`iso::witness::Iso<S, T>`](crate::iso::witness::iso::Iso) carries both directions as associated
//! functions on a witness type. That is an isomorphism stated as a pair of conversions; this module
//! exposes each direction as a [`Hom`] with named ends, so an isomorphism becomes what it is
//! mathematically — a bijective homomorphism — rather than a parallel concept.
//!
//! The existing traits are unchanged. Their witnesses gain map-shaped views.

use crate::iso::witness::iso::Iso;
use crate::iso::witness::ring_iso::RingIso as WitnessRingIso;
use crate::{Bijective, Hom, Injective, Isomorphism, RingHom, Surjective};
use core::marker::PhantomData;

/// The parameters a view is indexed by, in a form that carries no auto-trait obligation.
///
/// A bare `PhantomData<(W, S, T)>` would make the marker `Send`/`Sync` only when all three
/// parameters are, so a view of an isomorphism between non-`Send` types could not cross a thread.
/// A function type is `Send` and `Sync` for every `W`, `S`, `T`, with the same variance.
type Ends<W, S, T> = fn() -> (W, S, T);

/// The forward direction of an isomorphism witness, `S → T`, as a [`Hom`].
pub struct IsoForward<W, S, T>(PhantomData<Ends<W, S, T>>);

/// The backward direction of an isomorphism witness, `T → S`, as a [`Hom`].
pub struct IsoBackward<W, S, T>(PhantomData<Ends<W, S, T>>);

impl<W, S, T> IsoForward<W, S, T> {
    /// The forward view.
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<W, S, T> IsoBackward<W, S, T> {
    /// The backward view.
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<W, S, T> Hom for IsoForward<W, S, T>
where
    W: Iso<S, T>,
{
    type Domain = S;
    type Codomain = T;

    fn apply(&self, s: S) -> T {
        W::to_target(s)
    }
}

impl<W, S, T> Hom for IsoBackward<W, S, T>
where
    W: Iso<S, T>,
{
    type Domain = T;
    type Codomain = S;

    fn apply(&self, t: T) -> S {
        W::to_source(t)
    }
}

// An isomorphism is bijective in both directions: `to_source ∘ to_target` is the identity on `S`
// and `to_target ∘ to_source` the identity on `T`, which is exactly injectivity plus surjectivity.
impl<W, S, T> Injective for IsoForward<W, S, T> where W: Iso<S, T> {}
impl<W, S, T> Surjective for IsoForward<W, S, T> where W: Iso<S, T> {}
impl<W, S, T> Injective for IsoBackward<W, S, T> where W: Iso<S, T> {}
impl<W, S, T> Surjective for IsoBackward<W, S, T> where W: Iso<S, T> {}

// ...and each direction inverts the other, which is what `Bijective` promised existed.
impl<W, S, T> Isomorphism for IsoForward<W, S, T>
where
    W: Iso<S, T>,
{
    type Inverse = IsoBackward<W, S, T>;

    fn inverse(&self) -> Self::Inverse {
        IsoBackward::new()
    }
}

impl<W, S, T> Isomorphism for IsoBackward<W, S, T>
where
    W: Iso<S, T>,
{
    type Inverse = IsoForward<W, S, T>;

    fn inverse(&self) -> Self::Inverse {
        IsoForward::new()
    }
}

// A ring isomorphism preserves the ring structure in both directions, so both views are `RingHom`.
impl<W, S, T> RingHom for IsoForward<W, S, T>
where
    W: WitnessRingIso<S, T>,
    S: crate::Ring,
    T: crate::Ring,
{
}

impl<W, S, T> RingHom for IsoBackward<W, S, T>
where
    W: WitnessRingIso<S, T>,
    S: crate::Ring,
    T: crate::Ring,
{
}

/// Assert at compile time that a witness's forward view is a bijective homomorphism.
pub fn assert_iso_is_bijective_hom<W, S, T>()
where
    IsoForward<W, S, T>: Bijective,
{
}

// The derives would place `W: Debug`, `W: Default`, … bounds on these marker types because of the
// `PhantomData`. A view carries no data, so its parameters should not have to satisfy anything.
// The same reasoning picks the `Ends` alias for the `PhantomData` above.

impl<W, S, T> core::fmt::Debug for IsoForward<W, S, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("IsoForward")
    }
}

impl<W, S, T> core::fmt::Debug for IsoBackward<W, S, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("IsoBackward")
    }
}

impl<W, S, T> Clone for IsoForward<W, S, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<W, S, T> Copy for IsoForward<W, S, T> {}

impl<W, S, T> Clone for IsoBackward<W, S, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<W, S, T> Copy for IsoBackward<W, S, T> {}

impl<W, S, T> Default for IsoForward<W, S, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W, S, T> Default for IsoBackward<W, S, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<W, S, T> PartialEq for IsoForward<W, S, T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<W, S, T> Eq for IsoForward<W, S, T> {}

impl<W, S, T> PartialEq for IsoBackward<W, S, T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<W, S, T> Eq for IsoBackward<W, S, T> {}
