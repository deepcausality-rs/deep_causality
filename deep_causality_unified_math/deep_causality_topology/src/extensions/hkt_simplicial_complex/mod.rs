/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Chain;
use deep_causality_haft::{Foldable, Functor, HKT};
use deep_causality_linear::CsrMatrixWitness;
use std::marker::PhantomData;

/// # Why the element type carries no bound
///
/// `Chain<R, G>` carries no bound on its coefficient group `G`, and the categorical operations here move elements without
/// computing on them: `fmap` maps `A` to an unrelated `B`. Constraining the element type would
/// forbid mappings that are legitimate and work today, so the witness places no bound on the element type
/// rather than a placeholder. Operations that compute carry real trait bounds on the concrete
/// types. See `openspec/notes/archive/hkt_gat/hkt_gat_topology.md` §4.
///
/// # `fmap` preserves the complex
///
/// The complex is indexed by the precision parameter, which mapping the coefficients does not
/// touch, so it is carried across and its Hodge ⋆ operators survive. This used to be false: a
/// single parameter served both roles, `fmap` had to rebuild the complex with
/// `..Default::default()`, and the functor identity law failed for any complex carrying geometry.
pub struct ChainWitness<R>(PhantomData<R>);

impl<R> HKT for ChainWitness<R> {
    type Type<G> = Chain<R, G>;
}

// ----------------------------------------------------------------------------
// Functor
// ----------------------------------------------------------------------------

impl<R> Functor<ChainWitness<R>> for ChainWitness<R> {
    /// Maps the coefficients, carrying the complex across unchanged.
    ///
    /// The complex is indexed by the precision `R`, which this map does not touch, so it is cloned
    /// rather than rebuilt and its Hodge ⋆ operators survive. That is what makes `fmap(id, c) == c`
    /// hold. When the coefficient and the complex shared one parameter, `fmap` had to construct a
    /// `SimplicialComplex<B>` from nothing and dropped the geometry doing it.
    fn fmap<A, B, Func>(fa: Chain<R, A>, f: Func) -> Chain<R, B>
    where
        Func: FnMut(A) -> B,
    {
        let new_weights = <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(fa.weights, f);
        Chain::new(fa.complex, fa.grade, new_weights)
    }
}

// ----------------------------------------------------------------------------
// Foldable
// ----------------------------------------------------------------------------

impl<R> Foldable<ChainWitness<R>> for ChainWitness<R> {
    fn fold<A, B, Func>(fa: Chain<R, A>, init: B, f: Func) -> B
    where
        Func: FnMut(B, A) -> B,
    {
        // Re-use CsrMatrix foldable logic
        <CsrMatrixWitness as Foldable<CsrMatrixWitness>>::fold(fa.weights, init, f)
    }
}

// ----------------------------------------------------------------------------
// Adjunction
// ----------------------------------------------------------------------------
// Context: (Complex, Grade)

// No bound on `R`. Every operation here consumes the chain it descends into or shares the
// context's complex through `Arc`, so the precision parameter is never cloned.
// `Adjunction` is deliberately absent for this witness.
//
// It used to claim `Chain` is adjoint to itself, with `unit` building a one-entry chain of a
// one-entry chain and `counit` taking the first weight of the first inner chain. That pair cannot
// satisfy the defining bijection: `right_adjunct` after `left_adjunct` rebuilds a chain from a
// single stored weight, so it agrees with the original only when the chain had one entry to begin
// with. An `f` reading the whole chain, such as the sum of its weights, separates the two.
//
// The obstruction is structural, not a coding error: `unit` receives one value and must produce a
// chain over the whole complex, so everything but that one entry is invented. `StokesAdjunction`
// in `hkt_gauge::hkt_adjunction_stokes` is unaffected; it pairs two *different* functors,
// `ExteriorDerivativeWitness` with `BoundaryWitness`, which is the adjunction Stokes' theorem
// actually provides.
