/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Hypergraph;
use deep_causality_haft::{CoMonad, Functor, HKT};
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

/// # Why `NoConstraint`
///
/// `Hypergraph<T>` carries no element bound, and the categorical operations here move elements
/// without computing on them: `fmap` maps `A` to an unrelated `B`, and `extend` hands a cursor to a
/// closure. Constraining the element type would forbid mapping a hypergraph of labels to one of scores, which is legitimate and
/// works today. `NoConstraint` is the accurate statement, not a placeholder for a bound that
/// belongs here.
///
/// Operations that do compute on elements live on the concrete types and carry real trait bounds
/// there. See `openspec/notes/archive/hkt_gat/hkt_gat_topology.md` §4.
pub struct HypergraphWitness;

impl HKT for HypergraphWitness {
    type Type<T> = Hypergraph<T>;
}

impl Functor<HypergraphWitness> for HypergraphWitness {
    fn fmap<A, B, F>(fa: Hypergraph<A>, f: F) -> Hypergraph<B>
    where
        F: FnMut(A) -> B,
    {
        let new_data = CausalTensorWitness::fmap(fa.data, f);
        Hypergraph {
            num_nodes: fa.num_nodes,
            num_hyperedges: fa.num_hyperedges,
            incidence: fa.incidence,
            data: new_data,
            cursor: fa.cursor,
        }
    }
}

impl CoMonad<HypergraphWitness> for HypergraphWitness {
    fn extract<A>(fa: &Hypergraph<A>) -> A
    where
        A: Clone,
    {
        fa.data
            .as_slice()
            .get(fa.cursor)
            .cloned()
            .expect("Cursor OOB")
    }

    fn extend<A, B, Func>(fa: &Hypergraph<A>, mut f: Func) -> Hypergraph<B>
    where
        Func: FnMut(&Hypergraph<A>) -> B,
        A: Clone,
    {
        let size = fa.num_nodes;
        let shape = fa.data.shape().to_vec();
        let mut result_vec = Vec::with_capacity(size);

        for i in 0..size {
            let mut view = fa.clone_shallow();
            view.cursor = i;

            let val = f(&view);
            result_vec.push(val);
        }

        let new_data = CausalTensor::from_vec(result_vec, &shape);

        Hypergraph {
            num_nodes: fa.num_nodes,
            num_hyperedges: fa.num_hyperedges,
            incidence: fa.incidence.clone(),
            data: new_data,
            // Preserve the focus so `extend` satisfies the comonad laws (right
            // identity and associativity); resetting to `0` breaks them for a
            // non-zero focus.
            cursor: fa.cursor,
        }
    }
}
