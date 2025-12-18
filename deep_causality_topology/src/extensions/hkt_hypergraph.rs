/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Hypergraph;
use deep_causality_haft::{BoundedComonad, Functor, HKT};
use deep_causality_num::Zero;
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

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

impl BoundedComonad<HypergraphWitness> for HypergraphWitness {
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
        A: Zero + Copy + Clone,
        B: Zero + Copy + Clone,
    {
        let size = fa.num_nodes;
        let shape = fa.data.shape().to_vec(); // Preserve original shape
        let mut result_vec = Vec::with_capacity(size);

        for i in 0..size {
            let mut view = fa.clone_shallow();
            view.cursor = i;

            let val = f(&view);
            result_vec.push(val);
        }

        let new_data =
            CausalTensor::new(result_vec, shape).expect("Data tensor creation should succeed");

        Hypergraph {
            num_nodes: fa.num_nodes,
            num_hyperedges: fa.num_hyperedges,
            incidence: fa.incidence.clone(),
            data: new_data,
            cursor: 0,
        }
    }
}
