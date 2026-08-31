/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The symmetric-monoidal PROP: copy `Δ`, discard `ε`, merge `∇`, swap `σ`
//!
//! The generators of the symmetric-monoidal structure the effect graph is built in. The monoidal
//! product is the cartesian product (a pair `(A, B)`) with unit the terminal object `()`; the
//! carrier is any value, and — where merging is needed — any [`Monoid`].
//!
//! - **Copy comonoid `(Δ, ε)`.** In a cartesian category every object carries a *unique*
//!   cocommutative comonoid: the diagonal [`copy`](SymMonoidal::copy) `A → A ⊗ A` and the discard
//!   [`discard`](SymMonoidal::discard) `A → I` (T. Fox, "Coalgebras and Cartesian Categories,"
//!   *Comm. Algebra* 4(7), 1976). Coassociativity, counit, and cocommutativity therefore hold
//!   structurally.
//! - **Merge monoid `(∇, η)`.** The multiplication [`merge`](SymMonoidal::merge) `A ⊗ A → A` and
//!   unit [`unit`](SymMonoidal::unit) `I → A` are exactly a [`Monoid`]'s `combine`/`empty`, so the
//!   merge-monoid associativity and unit laws *are* the monoid laws. This `∇` is the substrate the
//!   deferred reconvergence-merge extension consumes — two effect branches fuse through `combine`.
//! - **Symmetry `σ`.** The braiding [`swap`](SymMonoidal::swap) `A ⊗ B → B ⊗ A` is its own inverse
//!   (`σ ∘ σ = id`), making the monoidal structure symmetric (Mac Lane, *CWM* §XI.1).
//! - **Copy–merge coherence.** `Δ` is a monoid homomorphism (`Δ(x ∇ y) = Δx ∇ Δy`, the bialgebra
//!   law), which holds for every monoid in a cartesian category; over a
//!   [`CommutativeMonoid`](deep_causality_algebra::CommutativeMonoid) `∇` is additionally invariant
//!   under `σ` (`∇ ∘ σ = ∇`).
//!
//! This module supplies the algebraic **substrate only**; the graph wiring that consumes `∇` for
//! branch reconvergence is out of scope here (it is the deferred reconvergence-merge extension).
//!
//! # Not to be confused with the endofunctor-level structure
//!
//! Everything here lives at the level of **values**: `Δ`, `ε`, `∇`, `η` and `σ` act on `A`, and the
//! product is the cartesian one. [`LaxMonoidal`](crate::LaxMonoidal) in `crate::lax_monoidal` is the
//! monoidal structure one level up, on **endofunctors**, and it is deliberately *not* cartesian —
//! it exists precisely to avoid requiring `Δ`. Both define a `unit`:
//! [`SymMonoidal::unit`] is the [`Monoid`] identity `M::empty()`, while `LaxMonoidal::unit` is
//! `η : I → F I`, the unit object lifted into a functor. They are unrelated maps that are both
//! reachable from the crate root, so name the level when you mean one of them.
//!
//! Laws are machine-checked in `lean/DeepCausalityFormal/Haft/SymmetricMonoidal.lean`
//! (`haft.monoidal.{comonoid_laws, merge_monoid_laws, symmetry}`) and witnessed in
//! `deep_causality_unified_math/deep_causality_haft/tests/formalization_lean/monoidal_tests.rs`.

use deep_causality_algebra::Monoid;

/// The generators of the cartesian symmetric-monoidal PROP: `copy`/`discard` (the copy comonoid),
/// `merge`/`unit` (the merge monoid), and `swap` (the symmetry). A zero-sized namespace — the
/// generators are ad-hoc polymorphic (over any value, or any [`Monoid`]), so they are associated
/// functions rather than trait methods.
pub struct SymMonoidal;

impl SymMonoidal {
    /// Copy `Δ`: the diagonal `A → A ⊗ A`.
    #[inline]
    pub fn copy<A: Clone>(a: A) -> (A, A) {
        (a.clone(), a)
    }

    /// Discard `ε`: the counit `A → I`, sending any value to the monoidal unit `()`.
    #[inline]
    pub fn discard<A>(_a: A) {}

    /// Swap `σ`: the symmetry `A ⊗ B → B ⊗ A`.
    #[inline]
    pub fn swap<A, B>((a, b): (A, B)) -> (B, A) {
        (b, a)
    }

    /// Merge `∇`: the monoid multiplication `A ⊗ A → A`.
    #[inline]
    pub fn merge<M: Monoid>((x, y): (M, M)) -> M {
        x.combine(y)
    }

    /// Unit `η`: the monoid identity `I → A`, the two-sided unit of [`merge`](SymMonoidal::merge).
    #[inline]
    pub fn unit<M: Monoid>() -> M {
        M::empty()
    }
}
