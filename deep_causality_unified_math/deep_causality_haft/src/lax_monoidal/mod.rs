/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # The lax monoidal structure on an endofunctor: `φ`, `η`, and the promise that they associate
//!
//! An applicative functor is a **monoid object**, in the same sense the algebra crate already uses
//! for `AddMonoid` and `MulMonoid`. Only the monoidal product differs: a monad is a monoid in
//! (End(𝒞), ∘, Id), an applicative one in (End(𝒞), ⊛_Day, Id). This module supplies the structure
//! maps of the second, at the level of **endofunctors**.
//!
//! Not to be confused with [`SymMonoidal`](crate::SymMonoidal) in `crate::monoidal`, which is the
//! *cartesian* symmetric-monoidal PROP at the level of **values**. Both modules define a `unit`,
//! and they are different maps:
//!
//! | | level | `unit` is |
//! |---|---|---|
//! | [`SymMonoidal`](crate::SymMonoidal) | values | `η : I → A`, the [`Monoid`](deep_causality_algebra::Monoid) identity `M::empty()` |
//! | [`LaxMonoidal`] | endofunctors | `η : I → F I`, the unit *object* lifted into `F` |
//!
//! # Why `pure` is not here
//!
//! A lax monoidal functor is `(F, φ, η)` with `η : I → F I` and `φ : F A ⊗ F B → F (A ⊗ B)`. The
//! Day-monoid presentation instead gives `η : Id ⇒ F`, whose component at `a` is `a -> F a`, which
//! is exactly [`Pure`](crate::Pure). Going from the first to the second needs the **diagonal**
//! `Δ : A → A ⊗ A`, because `pure(a) = fmap(η(), |()| a)` calls the constant function once per
//! slot.
//!
//! A category with a diagonal is cartesian; one with only `⊗` is merely monoidal. Haskell's is
//! cartesian, so the two presentations coincide there and the question never arises. Rust's move
//! semantics are not cartesian, and `Clone` is the diagonal — which this crate already says, in
//! [`SymMonoidal::copy`](crate::SymMonoidal::copy), whose signature is `copy<A: Clone>(a) -> (A, A)`
//! under a citation to T. Fox, "Coalgebras and Cartesian Categories," *Comm. Algebra* 4(7), 1976.
//!
//! So `pure` is a cartesian convenience rather than part of the monoid structure, and the traits
//! here require no `Clone` of anything. See
//! `openspec/notes/archive/hkt_gat/monoidal-applicative.md` for the measurements behind that.
//!
//! # Why `φ` and `η` are split across two traits
//!
//! Every context-carrying witness in this workspace has a lawful `φ` and no lawful `η`. `pure(a)`
//! at least receives a value; `unit()` receives nothing and must still name a complex, a grade, a
//! lattice or an adjacency map. Bundling them would exclude that whole family from a structure it
//! otherwise supports, and push it toward writing an unlawful `unit`. [`Semigroupal`] carries `φ`
//! alone, which is all [`MonoidalApplicative::apply`] needs; [`LaxMonoidal`] adds `η`, which is
//! needed only to *state* the unit laws.
//!
//! # Laws
//!
//! Machine-checked in `lean/DeepCausalityFormal/Haft/LaxMonoidal.lean` and witnessed in
//! `deep_causality_unified_math/deep_causality_haft/tests/formalization_lean/lax_monoidal_tests.rs`.
//!
//! Laws are stated for pure functions; a stateful `FnMut` closure voids them.

use crate::{Functor, HKT, Monad, Satisfies};

/// The semigroupal structure: the monoid multiplication `φ` on its own, with no unit.
///
/// `zip_with` is the required method and `zip` is derived from it, not the other way round. That
/// ordering is deliberate. Deriving [`apply`](MonoidalApplicative::apply) through `zip` builds an
/// `F::Type<(Func, A)>` and hands it to [`fmap`](Functor::fmap), so the *tuple* would have to
/// satisfy the witness constraint:
///
/// ```text
/// error[E0277]: the trait bound `(Func, A): Satisfies<<F as HKT>::Constraint>` is not satisfied
/// note: required by a bound in `fmap`
/// ```
///
/// That bound then leaks: every function generic over the witness has to restate it or fail at the
/// call site with the same error. `zip_with` never constructs the tuple, so `apply` and its callers
/// are free of it, and only `zip` carries it, where a tuple is what the caller actually wanted.
///
/// The same shape appears twice elsewhere in the workspace: [`MonoidalMerge::merge`] at the
/// `HKT3Unbound` level, and `LatticeGaugeFieldWitness::zip_with` concretely in
/// `deep_causality_topology`, which returns `Result` because its `φ` is partial.
///
/// # Laws
///
/// 1. **Naturality**: `zip(fmap(fa, f), fmap(fb, g)) == fmap(zip(fa, fb), |(a, b)| (f(a), g(b)))`
/// 2. **Associativity**: `zip(zip(fa, fb), fc) ≅ zip(fa, zip(fb, fc))`, modulo the associator
///    `((A, B), C) ≅ (A, (B, C))`
///
/// Associativity is a *promise*, not a consequence of the signature. A witness records it by
/// implementing [`Convolutional`].
///
/// [`MonoidalMerge::merge`]: crate::MonoidalMerge::merge
pub trait Semigroupal<F: HKT>: Functor<F> {
    /// `φ` followed by the payload map, in one step: the structure map, fused.
    ///
    /// Pairs `fa` with `fb` position by position and combines each pair with `f`. Nothing is
    /// consumed twice, so no payload needs `Clone`.
    fn zip_with<A, B, C, Func>(fa: F::Type<A>, fb: F::Type<B>, f: Func) -> F::Type<C>
    where
        A: Satisfies<F::Constraint>,
        B: Satisfies<F::Constraint>,
        C: Satisfies<F::Constraint>,
        Func: FnMut(A, B) -> C;

    /// `φ : F A ⊗ F B → F (A ⊗ B)`, derived from [`zip_with`](Semigroupal::zip_with).
    ///
    /// Carries the `(A, B): Satisfies<F::Constraint>` bound that `zip_with` avoids, because this
    /// is the operation that actually builds the tuple.
    fn zip<A, B>(fa: F::Type<A>, fb: F::Type<B>) -> F::Type<(A, B)>
    where
        A: Satisfies<F::Constraint>,
        B: Satisfies<F::Constraint>,
        (A, B): Satisfies<F::Constraint>,
    {
        Self::zip_with(fa, fb, |a, b| (a, b))
    }
}

/// The full lax monoidal structure: [`Semigroupal`]'s `φ` plus the unit `η`.
///
/// # When a witness must *not* implement this
///
/// `unit` takes no argument and must still return an inhabited `F::Type<()>`. A witness whose
/// carrier needs a simplicial complex, a grade, a lattice, a shape or an adjacency map cannot
/// supply one without inventing it, and an invented context does not satisfy the unit laws: it
/// fails at every real value rather than in some corner. Such a witness implements
/// [`Semigroupal`] alone.
///
/// This is not hypothetical. A deleted `GaugeField` [`MonoidalMerge`] impl in this workspace did
/// fabricate its unit, and five tests were written that asserted the resulting defect as the
/// specification.
///
/// # Laws
///
/// In addition to [`Semigroupal`]'s:
///
/// 3. **Left unit**: `fmap(zip(unit(), fa), |((), a)| a) == fa`
/// 4. **Right unit**: `fmap(zip(fa, unit()), |(a, ())| a) == fa`
///
/// [`MonoidalMerge`]: crate::MonoidalMerge
pub trait LaxMonoidal<F: HKT>: Semigroupal<F> {
    /// `η : I → F I`. The unit *object* `()` lifted into `F`.
    ///
    /// Distinct from [`SymMonoidal::unit`](crate::SymMonoidal::unit), which is a
    /// [`Monoid`](deep_causality_algebra::Monoid) identity at the level of values, and from
    /// [`Pure::pure`](crate::Pure::pure), which is `a -> F a` and needs the diagonal.
    fn unit() -> F::Type<()>;
}

/// Marker. Promises that `μ` associates under **composition**: a monoid object in (End(𝒞), ∘, Id).
///
/// The promise is the monad associativity law,
/// `bind(bind(m, f), g) == bind(m, |x| bind(f(x), g))`. The compiler cannot check it, so
/// implementing this trait is an assertion by the developer.
///
/// # Never handed out by inference
///
/// This trait SHALL NOT be blanket-implemented, derived, or implied by any other trait. Each impl
/// is one line naming one witness, following
/// [`Associative`](deep_causality_algebra::Associative), whose documentation records why a marker
/// carrying an unverifiable promise cannot be granted by inference: a downstream type would
/// silently acquire a law nobody promised. The *absence* of this marker on a witness is therefore
/// readable as a deliberate withholding.
///
/// # Holding both markers
///
/// A witness carrying both this and [`Convolutional`] owes the applicative-monad coherence law,
/// `apply(f_ab, f_a) == bind(f_ab, |f| fmap(f_a, f))`, proved as `haft.monad.applicative_coherence`
/// and discharged by a law test naming the witness.
pub trait Compositional<F: HKT>: Monad<F> {}

/// Marker. Promises that `φ` associates under **Day convolution**: a monoid object in
/// (End(𝒞), ⊛, Id).
///
/// The promise is `zip(zip(a, b), c) ≅ zip(a, zip(b, c))` up to reassociation, together with the
/// naturality of `φ`. The compiler cannot check either, so implementing this trait is an assertion
/// by the developer.
///
/// Carries the same no-inference discipline as [`Compositional`], and the same coherence obligation
/// when a witness holds both.
pub trait Convolutional<F: HKT>: Semigroupal<F> {}

/// The applicative structure that comes from the monoid: `apply` derived from `φ`, free of the
/// diagonal.
///
/// # Why this is a sibling of [`Applicative`](crate::Applicative) rather than a replacement
///
/// There are two routes to an applicative and they differ exactly on whether they need `Clone`.
/// The monoidal route pairs slot with slot and consumes nothing twice. The monadic route induces
/// `ap(ff, fa) = bind(ff, |f| fmap(fa, f))`, which re-runs the continuation once per function and
/// therefore consumes `fa` many times.
///
/// `VecWitness` is on the second route and cannot move: its `apply` is the cartesian list
/// applicative, pinned as the only lawful choice by `haft.monad.applicative_coherence`, and it
/// needs `Func: Clone`. `ZipList` is not an escape, because the unit of a positional zip is the
/// infinite repeat and a finite `Vec` cannot represent it. So [`Applicative`](crate::Applicative)
/// keeps its signature, its `A: Clone` bound and all of its impls, and this trait sits beside it.
/// A witness may hold both, and then owes a law test that the two `apply`s agree.
///
/// # Laws
///
/// Those of [`Semigroupal`] and, where the witness is also [`LaxMonoidal`], the unit laws. The
/// four McBride–Paterson laws remain stated on [`Applicative`](crate::Applicative); they are not
/// restated here, because the monoid coherence conditions alone do not pin a witness's applicative
/// (both the function-major and argument-major cartesian products on `Vec` satisfy all of them).
/// # The gate is load-bearing
///
/// A witness carrying the full structure but withholding the promise cannot reach `apply`.
///
/// ```compile_fail
/// use deep_causality_haft::{
///     Convolutional, Functor, HKT, MonoidalApplicative, NoConstraint, Satisfies, Semigroupal,
/// };
///
/// pub struct Unpromised;
/// impl HKT for Unpromised {
///     type Constraint = NoConstraint;
///     type Type<T> = Vec<T>;
/// }
/// impl Functor<Unpromised> for Unpromised {
///     fn fmap<A, B, Func>(fa: Vec<A>, f: Func) -> Vec<B>
///     where
///         A: Satisfies<NoConstraint>,
///         B: Satisfies<NoConstraint>,
///         Func: FnMut(A) -> B,
///     {
///         fa.into_iter().map(f).collect()
///     }
/// }
/// impl Semigroupal<Unpromised> for Unpromised {
///     fn zip_with<A, B, C, Func>(fa: Vec<A>, fb: Vec<B>, mut f: Func) -> Vec<C>
///     where
///         Func: FnMut(A, B) -> C,
///     {
///         fa.into_iter().zip(fb).map(|(a, b)| f(a, b)).collect()
///     }
/// }
///
/// // No `impl Convolutional<Unpromised> for Unpromised {}` — the promise is withheld, so this
/// // fails with an unsatisfied `Convolutional` bound.
/// impl MonoidalApplicative<Unpromised> for Unpromised {}
/// ```
///
/// Adding the withheld line makes the same code compile:
///
/// ```rust
/// use deep_causality_haft::{
///     Convolutional, Functor, HKT, MonoidalApplicative, NoConstraint, Satisfies, Semigroupal,
/// };
///
/// pub struct Promised;
/// impl HKT for Promised {
///     type Constraint = NoConstraint;
///     type Type<T> = Vec<T>;
/// }
/// impl Functor<Promised> for Promised {
///     fn fmap<A, B, Func>(fa: Vec<A>, f: Func) -> Vec<B>
///     where
///         A: Satisfies<NoConstraint>,
///         B: Satisfies<NoConstraint>,
///         Func: FnMut(A) -> B,
///     {
///         fa.into_iter().map(f).collect()
///     }
/// }
/// impl Semigroupal<Promised> for Promised {
///     fn zip_with<A, B, C, Func>(fa: Vec<A>, fb: Vec<B>, mut f: Func) -> Vec<C>
///     where
///         Func: FnMut(A, B) -> C,
///     {
///         fa.into_iter().zip(fb).map(|(a, b)| f(a, b)).collect()
///     }
/// }
/// impl Convolutional<Promised> for Promised {}
/// impl MonoidalApplicative<Promised> for Promised {}
///
/// let add: fn(i32) -> i32 = |x| x + 10;
/// assert_eq!(Promised::apply(vec![add, add], vec![1, 2]), vec![11, 12]);
/// ```
pub trait MonoidalApplicative<F: HKT>: Functor<F> + Convolutional<F> {
    /// Applies a function in context to an argument in context, through `φ`.
    ///
    /// Unlike [`Applicative::apply`](crate::Applicative::apply) this requires no `Clone` on `A`:
    /// `zip_with` pairs each function with its argument exactly once.
    fn apply<A, B, Func>(ff: F::Type<Func>, fa: F::Type<A>) -> F::Type<B>
    where
        A: Satisfies<F::Constraint>,
        B: Satisfies<F::Constraint>,
        Func: Satisfies<F::Constraint> + FnMut(A) -> B,
    {
        Self::zip_with(ff, fa, |mut f, a| f(a))
    }
}
