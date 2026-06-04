/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! The value-level `Arrow` algebra: a strong category for wiring operators together.
//!
//! Where [`Morphism`](crate::Morphism) is the *witness-level* interface (identity +
//! application, but no composition under the no-`dyn` policy), this module provides the
//! *value-level* algebra where composition is **total**: every combinator returns a new
//! concrete `Arrow`-implementing type, so composites compose.
//!
//! - **Category:** [`Id`], [`Lift`] (lift a function), [`Compose`] (`f >>> g`).
//! - **Strength (the monoidal product):** [`First`], [`Second`], [`Split`] (`***`),
//!   [`Fanout`] (`&&&`) — the operation the causal monad's `bind` cannot express.
//! - **Builder:** [`arrow`] + [`ArrowBuilder`] hide the combinator types behind a fluent
//!   chain (the textual form of a wiring diagram).

mod builder;
mod compose;
mod fanout;
mod first;
mod id;
mod lift;
mod second;
mod split;

pub use builder::{ArrowBuilder, arrow};
pub use compose::Compose;
pub use fanout::Fanout;
pub use first::First;
pub use id::Id;
pub use lift::Lift;
pub use second::Second;
pub use split::Split;

/// A value-level arrow `In → Out`: a runnable, composable transformation.
///
/// # Category Theory
///
/// `Arrow` is a **strong category** (Hughes' Arrow): `Id`/`Compose` give the category,
/// the `first`/`split` combinators give the monoidal **strength**, and `fanout` gives the
/// diagonal. Unlike the witness-level [`Morphism`](crate::Morphism) — which cannot host
/// composition, because composing two closures yields an unnameable type and `Box<dyn Fn>`
/// is forbidden — every combinator here returns a *new concrete type*, so **composition is
/// total** and everything is monomorphized (zero-cost, no `dyn`, no macros).
///
/// The combinator methods are provided; an implementor supplies only `In`, `Out`, and
/// `run`. `run` takes `&self`, so an arrow is reusable.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not an `Arrow`",
    note = "lift a function with `Lift::new(f)` (or the `arrow(f)` builder), or implement `Arrow` for your operator type"
)]
pub trait Arrow {
    /// The input object the arrow consumes.
    type In;
    /// The output object the arrow produces.
    type Out;

    /// Apply the arrow to an input.
    fn run(&self, input: Self::In) -> Self::Out;

    /// Sequential composition `f >>> g`: run `self`, then `g` on its output.
    #[inline]
    fn compose<G>(self, g: G) -> Compose<Self, G>
    where
        Self: Sized,
        G: Arrow<In = Self::Out>,
    {
        Compose::new(self, g)
    }

    /// `first`: lift `A → B` to `(A, C) → (B, C)`, passing the second component through.
    #[inline]
    fn first<C>(self) -> First<Self, C>
    where
        Self: Sized,
    {
        First::new(self)
    }

    /// `second`: lift `A → B` to `(C, A) → (C, B)`, passing the first component through.
    #[inline]
    fn second<C>(self) -> Second<Self, C>
    where
        Self: Sized,
    {
        Second::new(self)
    }

    /// The monoidal product `***`: run `self` and `g` in parallel on a pair —
    /// `(A, C) → (B, D)` from `self: A → B` and `g: C → D`.
    #[inline]
    fn split<G>(self, g: G) -> Split<Self, G>
    where
        Self: Sized,
        G: Arrow,
    {
        Split::new(self, g)
    }

    /// Fanout `&&&`: feed one input to two arrows — `A → (B, C)` from `self: A → B` and
    /// `g: A → C`. Requires `In: Clone`.
    #[inline]
    fn fanout<G>(self, g: G) -> Fanout<Self, G>
    where
        Self: Sized,
        G: Arrow<In = Self::In>,
        Self::In: Clone,
    {
        Fanout::new(self, g)
    }
}
