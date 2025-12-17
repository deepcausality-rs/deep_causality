/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::Graph;
use deep_causality_haft::{BoundedComonad, Functor, HKT};
use deep_causality_num::Zero;
use deep_causality_tensor::{CausalTensor, CausalTensorWitness};

pub struct GraphWitness;

impl HKT for GraphWitness {
    type Type<T> = Graph<T>;
}

impl Functor<GraphWitness> for GraphWitness {
    fn fmap<A, B, F>(fa: Graph<A>, f: F) -> Graph<B>
    where
        F: FnMut(A) -> B,
    {
        let new_data = CausalTensorWitness::fmap(fa.data, f);
        Graph {
            num_vertices: fa.num_vertices,
            adjacencies: fa.adjacencies,
            num_edges: fa.num_edges,
            data: new_data,
            cursor: fa.cursor,
        }
    }
}

impl BoundedComonad<GraphWitness> for GraphWitness {
    fn extract<A>(fa: &Graph<A>) -> A
    where
        A: Clone,
    {
        fa.data
            .as_slice()
            .get(fa.cursor)
            .cloned()
            .expect("Cursor OOB")
    }

    fn extend<A, B, Func>(fa: &Graph<A>, mut f: Func) -> Graph<B>
    where
        Func: FnMut(&Graph<A>) -> B,
        A: Zero + Copy + Clone,
        B: Zero + Copy + Clone,
    {
        let size = fa.num_vertices;
        // Since Graph data is typically 1D (per vertex), but could be multi-dimensional per vertex?
        // Wait, Graph definition uses num_vertices for size.
        // Let's check if Graph has .data with shape.
        // Graph struct has `data: CausalTensor<T>`.
        // So we should use fa.data.shape().to_vec().
        let shape = fa.data.shape().to_vec();
        let mut result_vec = Vec::with_capacity(size);

        for i in 0..size {
            let mut view = fa.clone_shallow();
            view.cursor = i;

            let val = f(&view);
            result_vec.push(val);
        }

        let new_data =
            CausalTensor::new(result_vec, shape).expect("Data tensor creation should succeed");

        Graph {
            num_vertices: fa.num_vertices,
            adjacencies: fa.adjacencies.clone(),
            num_edges: fa.num_edges,
            data: new_data,
            cursor: 0,
        }
    }
}
