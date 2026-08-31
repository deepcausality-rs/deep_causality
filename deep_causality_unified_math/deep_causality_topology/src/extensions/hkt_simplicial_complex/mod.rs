/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::errors::topology_error::{TopologyError, TopologyErrorEnum};
use crate::{Chain, SimplicialComplex};
use deep_causality_haft::{Adjunction, Foldable, Functor, HKT, NoConstraint, Pure, Satisfies};
use deep_causality_linear::CsrMatrixWitness;
use std::marker::PhantomData;
use std::sync::Arc;

/// # Why `NoConstraint`
///
/// `Chain<R, G>` carries no bound on its coefficient group `G`, and the categorical operations here move elements without
/// computing on them: `fmap` maps `A` to an unrelated `B`. Constraining the element type would
/// forbid mappings that are legitimate and work today, so `NoConstraint` is the accurate statement
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
    type Constraint = NoConstraint;
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
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
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
        A: Satisfies<NoConstraint>,
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
impl<R> Adjunction<ChainWitness<R>, ChainWitness<R>, (Arc<SimplicialComplex<R>>, usize)>
    for ChainWitness<R>
{
    type Error = TopologyError;

    fn unit<A>(ctx: &(Arc<SimplicialComplex<R>>, usize), a: A) -> Chain<R, Chain<R, A>>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
        // We remove unnecessary recursive bounds if possible.
    {
        let (complex, grade) = ctx;

        // Both chains are indexed by the same precision `R`, so the context's complex is shared
        // rather than rebuilt. It used to be reconstructed twice with `..Default::default()`, once
        // per nesting level, which dropped the Hodge ⋆ operators both times.
        let inner_weights = <CsrMatrixWitness as Pure<CsrMatrixWitness>>::pure(a);
        let inner_chain = Chain::new(Arc::clone(complex), *grade, inner_weights);

        let outer_weights = <CsrMatrixWitness as Pure<CsrMatrixWitness>>::pure(inner_chain);
        Chain::new(Arc::clone(complex), *grade, outer_weights)
    }

    /// # Errors
    ///
    /// Returns [`TopologyError`] when the outer chain stores no value, or when the inner chain it
    /// holds stores none. CSR drops explicit zeros, so an all-zero chain is empty and reachable.
    fn counit<B>(
        _ctx: &(Arc<SimplicialComplex<R>>, usize),
        lrb: Chain<R, Chain<R, B>>,
    ) -> Result<B, Self::Error>
    where
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
    {
        // counit: Chain<Chain<B>> -> B
        //
        // `lrb` is owned, so both levels are taken apart rather than cloned. Cloning the inner
        // `Chain<R, B>` would need `R: Clone` for a value that is discarded on the next line.
        let (_, _, outer_values, _) = lrb.weights.into_parts();
        let inner_chain = outer_values.into_iter().next().ok_or_else(|| {
            TopologyError(TopologyErrorEnum::InvalidInput(
                "Adjunction::counit: the outer chain stores no value, so there is no inner chain \
                 to descend into"
                    .into(),
            ))
        })?;

        let (_, _, inner_values, _) = inner_chain.weights.into_parts();
        inner_values.into_iter().next().ok_or_else(|| {
            TopologyError(TopologyErrorEnum::InvalidInput(
                "Adjunction::counit: the inner chain stores no value, so there is no B to return"
                    .into(),
            ))
        })
    }

    fn left_adjunct<A, B, F>(ctx: &(Arc<SimplicialComplex<R>>, usize), a: A, f: F) -> Chain<R, B>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        F: FnMut(Chain<R, A>) -> B,
    {
        // left: a -> f(unit(a))
        let wrapped = Self::unit(ctx, a);
        Self::fmap(wrapped, f)
    }

    /// The right adjunct `(A -> Chain<B>) -> (Chain<A> -> B)`.
    ///
    /// # Errors
    ///
    /// Returns [`TopologyError`] on a chain storing nothing, in either of two places, and the
    /// message says which: `la` stores no value, so there is no `A` to apply `f` to, or the chain
    /// `f` returns stores none, so there is no `B` to hand back.
    ///
    /// `B` carries no `Default`, so there is no value to return when there is nothing to return,
    /// and fabricating one would be a lie about which element the adjunct selected. A chain is
    /// empty when its weight matrix stores no entries, which `CsrMatrix::new()` produces and which
    /// dropping an explicit zero also produces, so both arms are reachable and both are tested.
    fn right_adjunct<A, B, F>(
        _ctx: &(Arc<SimplicialComplex<R>>, usize),
        la: Chain<R, A>,
        f: F,
    ) -> Result<B, Self::Error>
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
        F: FnMut(A) -> Chain<R, B>,
    {
        // right: (A -> R<B>) -> (L<A> -> B)
        let result_chain: Chain<R, Chain<R, B>> = Self::fmap(la, f);
        let (_, _, outer_values, _) = result_chain.weights.into_parts();

        let inner_chain = outer_values.into_iter().next().ok_or_else(|| {
            TopologyError(TopologyErrorEnum::InvalidInput(
                "Adjunction::right_adjunct was called on a Chain that stores nothing, so there \
                 is no A to apply f to"
                    .into(),
            ))
        })?;

        let (_, _, inner_values, _) = inner_chain.weights.into_parts();
        inner_values.into_iter().next().ok_or_else(|| {
            TopologyError(TopologyErrorEnum::InvalidInput(
                "Adjunction::right_adjunct: f returned a Chain that stores nothing, so there is \
                 no B to return"
                    .into(),
            ))
        })
    }
}
