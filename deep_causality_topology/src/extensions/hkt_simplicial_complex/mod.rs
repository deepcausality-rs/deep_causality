/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::{Chain, SimplicialComplex};
use deep_causality_haft::{
    Adjunction, CoMonad, Foldable, Functor, HKT, NoConstraint, Pure, Satisfies,
};
use deep_causality_sparse::CsrMatrixWitness;
use std::sync::Arc;

pub struct ChainWitness;

impl HKT for ChainWitness {
    type Constraint = NoConstraint;
    type Type<T>
        = Chain<T>
    where
        T: Satisfies<NoConstraint>;
}

// ----------------------------------------------------------------------------
// Functor
// ----------------------------------------------------------------------------
impl Functor<ChainWitness> for ChainWitness {
    fn fmap<A, B, Func>(fa: Chain<A>, f: Func) -> Chain<B>
    where
        A: Satisfies<NoConstraint>,
        B: Satisfies<NoConstraint>,
        Func: FnMut(A) -> B,
    {
        // Re-use CsrMatrix functor logic on weights
        let new_weights = <CsrMatrixWitness as Functor<CsrMatrixWitness>>::fmap(fa.weights, f);

        Chain::new(fa.complex, fa.grade, new_weights)
    }
}

// ----------------------------------------------------------------------------
// Foldable
// ----------------------------------------------------------------------------
impl Foldable<ChainWitness> for ChainWitness {
    fn fold<A, B, Func>(fa: Chain<A>, init: B, f: Func) -> B
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
impl Adjunction<ChainWitness, ChainWitness, (Arc<SimplicialComplex>, usize)> for ChainWitness {
    fn unit<A>(ctx: &(Arc<SimplicialComplex>, usize), a: A) -> Chain<Chain<A>>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
    {
        let (complex, grade) = ctx;

        // Inner Chain: contains 'a' at pos 0
        let inner_weights = <CsrMatrixWitness as Pure<CsrMatrixWitness>>::pure(a);
        let inner_chain = Chain::new(complex.clone(), *grade, inner_weights);

        // Outer Chain: contains inner_chain at pos 0
        let outer_weights = <CsrMatrixWitness as Pure<CsrMatrixWitness>>::pure(inner_chain);
        Chain::new(complex.clone(), *grade, outer_weights)
    }

    fn counit<B>(_ctx: &(Arc<SimplicialComplex>, usize), lrb: Chain<Chain<B>>) -> B
    where
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
    {
        // counit: Chain<Chain<B>> -> B
        // Extract via CsrMatrix CoMonad extract.
        // Requires Clone on B.
        let inner_chain = <CsrMatrixWitness as CoMonad<CsrMatrixWitness>>::extract(&lrb.weights);
        <CsrMatrixWitness as CoMonad<CsrMatrixWitness>>::extract(&inner_chain.weights)
    }

    fn left_adjunct<A, B, F>(ctx: &(Arc<SimplicialComplex>, usize), a: A, f: F) -> Chain<B>
    where
        A: Satisfies<NoConstraint> + Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint>,
        F: Fn(Chain<A>) -> B,
    {
        // left: a -> f(unit(a))
        let wrapped = Self::unit(ctx, a);
        Self::fmap(wrapped, f)
    }

    fn right_adjunct<A, B, F>(_ctx: &(Arc<SimplicialComplex>, usize), la: Chain<A>, f: F) -> B
    where
        A: Satisfies<NoConstraint> + Clone,
        B: Satisfies<NoConstraint> + Satisfies<NoConstraint>,
        F: FnMut(A) -> Chain<B>,
    {
        // right: (A -> R<B>) -> (L<A> -> B)
        // map la with f -> Chain<Chain<B>>
        // then counit
        // Implementation must avoid `B: Clone`.
        // We calculate Chain<Chain<B>>.
        let result_chain: Chain<Chain<B>> = Self::fmap(la, f);

        // We unpack result_chain manually to avoid CoMonad::extract (which requires Clone).
        // Since we own result_chain, we can consume it.
        // Unpack outer weights.
        let (_, _, outer_values, _) = result_chain.weights.into_parts();

        // Get first inner chain
        if let Some(inner_chain) = outer_values.into_iter().next() {
            // Unpack inner weights
            let (_, _, inner_values, _) = inner_chain.weights.into_parts();
            // Get first B
            if let Some(val) = inner_values.into_iter().next() {
                return val;
            }
        }

        panic!("Adjunction::right_adjunct resulted in empty chain.");
    }
}
