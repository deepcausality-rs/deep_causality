/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Topology;
use deep_causality_haft::{CoMonad, Functor, HKT};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};
use std::marker::PhantomData;

/// # Why the element type carries no bound
///
/// `Topology<R, G>` carries no bound on its coefficient group `G`, and the categorical operations here move elements without
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
pub struct TopologyWitness<R>(PhantomData<R>);

impl<R> HKT for TopologyWitness<R> {
    type Type<G> = Topology<R, G>;
}

impl<R> Functor<TopologyWitness<R>> for TopologyWitness<R> {
    /// Maps the coefficients, carrying the complex across unchanged.
    ///
    /// The complex is indexed by the precision `R`, which this map does not touch, so it is shared
    /// rather than rebuilt and its Hodge ⋆ operators survive. When the coefficient and the complex
    /// shared one parameter, `fmap` had to construct a `SimplicialComplex<B>` from nothing and
    /// dropped the geometry doing it, which broke `fmap(id, t) == t`.
    fn fmap<A, B, F>(fa: Topology<R, A>, f: F) -> Topology<R, B>
    where
        F: FnMut(A) -> B,
    {
        let new_data = CausalTensorWitness::fmap(fa.data, f);

        Topology {
            complex: fa.complex,
            grade: fa.grade,
            data: new_data,
            cursor: fa.cursor,
        }
    }
}

impl<R: Clone> CoMonad<TopologyWitness<R>> for TopologyWitness<R> {
    fn extract<A>(fa: &Topology<R, A>) -> A
    where
        A: Clone,
    {
        fa.data
            .as_slice()
            .get(fa.cursor)
            .cloned()
            .expect("Cursor OOB")
    }

    fn extend<A, B, Func>(fa: &Topology<R, A>, mut f: Func) -> Topology<R, B>
    where
        Func: FnMut(&Topology<R, A>) -> B,
        A: Clone,
    {
        let size = fa.data.len();
        let shape = fa.data.shape().to_vec();
        let mut result_vec = Vec::with_capacity(size);

        for i in 0..size {
            let mut view = fa.clone_shallow();
            view.cursor = i;

            let val = f(&view);
            result_vec.push(val);
        }

        Topology {
            // Shared, not rebuilt: the geometry belongs to `R` and `extend` does not touch it.
            complex: fa.complex.clone(),
            grade: fa.grade,
            data: CausalTensor::from_vec(result_vec, &shape),
            // Preserve the focus so `extend` satisfies the comonad laws (right
            // identity and associativity); resetting to `0` breaks them for a
            // non-zero focus.
            cursor: fa.cursor,
        }
    }
}
