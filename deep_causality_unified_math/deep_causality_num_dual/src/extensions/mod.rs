/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! HKT witness for the dual number type.
//!
//! # What this witness implements
//!
//! `Functor`, `Foldable`, and the lax monoidal stack: `Semigroupal`, `LaxMonoidal`,
//! `Convolutional` and `MonoidalApplicative`. Not `Pure`, not `Applicative`, not `Monad`, and
//! deliberately not `CoMonad`.
//!
//! # This functor is not automatic differentiation
//!
//! `fmap` maps `re` and `du` **independently**. That is the pair functor, and it carries no chain
//! rule: `fmap(x, f)` is not `f` differentiated at `x`. The chain rule lives where it always did,
//! in the arithmetic and elementary-function impls that name `T: Real`, and those are untouched.
//!
//! What the functor layer is for is structural: traversing a dual's two channels uniformly,
//! migrating a computation between precisions (`Dual<f32>` to `Dual<f64>`), or relabelling
//! components into a type that carries no arithmetic at all. Reach for the arithmetic if you want a
//! derivative.
//!
//! # Why `Pure`, `Applicative` and `Monad` are absent
//!
//! `Dual<A>` is a two-slot product, `A^S` for the two-element index set. Constructing one needs two
//! values. `Pure::pure` receives one, by value, bounded only by `Satisfies`, an empty marker; an
//! impl cannot add `Clone` (E0276) and a constraint marker cannot supply it (E0599). Filling both
//! slots from one moved value is the diagonal `Δ : A → A ⊗ A`, which in Rust *is* `Clone`. Since
//! `Applicative<F>: Functor<F> + Pure<F>` and `Monad` names `Pure` too, both are out of reach for
//! the same reason. [`MonoidalApplicative`] reaches `apply` without them, because `zip_with` pairs
//! slot with slot and never invokes the diagonal.
//!
//! # Why `CoMonad` is deferred
//!
//! Not an oversight. `unified_math_gaps.md` §4.1 item E2 asked for it, and the request is
//! deliberately not met, for two reasons.
//!
//! There is no forced answer. Lawful comonads on `A^S` whose `extract` evaluates at a fixed
//! identity correspond to monoid structures on `S`, and a two-element set with a chosen identity
//! carries exactly two. Both satisfy all four comonad laws:
//!
//! - **swap**, from the group ℤ/2 (`du · du = re`):
//!   `duplicate(w) = Dual { re: w, du: Dual { re: w.du, du: w.re } }`
//! - **absorbing**, from the idempotent monoid (`du · du = du`):
//!   `duplicate(w) = Dual { re: w, du: Dual { re: w.du, du: w.du } }`
//!
//! A third shape that looks natural, `duplicate(w) = Dual { re: w, du: w }`, is *unlawful*: it
//! fails the counit law, since `extend(w, extract)` then returns `Dual { re: w.re, du: w.re }`
//! rather than `w`.
//!
//! And there is no caller. On a tensor or a manifold the comultiplication buys stencils and local
//! field operations, which is why `CausalTensorWitness::extend` rotates through a shifted view and
//! `ManifoldWitness::extend` walks a cursor over a complex. On a two-slot product `extend` computes
//! something per channel that can see both channels, and nothing in this workspace wants that.
//! Choosing one of two arbitrary-but-lawful structures, with no consumer to validate the choice
//! against, would lock a default into the public API that later code would come to depend on.
//!
//! Should the deferral be lifted, prefer **absorbing**. **swap** makes `extend` observe a dual
//! whose derivative channel holds a function value, which carries no meaning in forward-mode AD
//! terms. The decision is recorded in
//! `openspec/changes/add-lax-monoidal-applicative/design.md`.
//!
//! # Why `NoConstraint`
//!
//! These operations move components; they never compute with them. `fmap` maps `Dual<A>` to
//! `Dual<B>` for unrelated `A` and `B`, and `zip_with` needs no arithmetic either. Everything that
//! computes names `T: Real` on its own impl, which the compiler enforces and no witness can bypass.
//!
//! [`MonoidalApplicative`]: deep_causality_haft::MonoidalApplicative

pub mod hkt_dual;

pub use hkt_dual::DualWitness;
