/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) "2025" . The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use crate::types::graph::Graph;
use deep_causality_num::RealField;
use deep_causality_tensor::CausalTensorError;
use deep_causality_tensor::LinearAlgebraBackend;
use deep_causality_tensor::TensorBackend;
use deep_causality_tensor::TensorData;

/// An accelerated view of a topology structure.
/// Projects a Graph/HyperGraph into an Adjacency Matrix on the Backend.
pub struct TopologyView<B: TensorBackend, T: TensorData> {
    /// Adjacency Matrix (Shape: [N, N])
    pub matrix: <B as TensorBackend>::Tensor<T>,

    /// Node Degrees Vector (Shape: [N])
    pub degrees: <B as TensorBackend>::Tensor<T>,

    /// Mapping from Matrix Index to Node ID
    pub index_map: Vec<usize>,
}

impl<B: LinearAlgebraBackend, T: TensorData + RealField + From<f32>> TopologyView<B, T> {
    /// Projects a Graph into a TensorBackend Adjacency Matrix.
    /// Graph uses usize as node identifiers.
    pub fn from_graph<M>(graph: &Graph<M>) -> Self
    where
        M: Clone, // Metadata type
    {
        // Graph API access via getters
        // num_vertices(), adjacencies() -> &BTreeMap<usize, Vec<usize>>
        let adj_map = graph.adjacencies();
        let n = graph.num_vertices();

        let mut index_map = Vec::with_capacity(n);
        let mut id_to_idx = std::collections::HashMap::with_capacity(n);

        // We need a stable ordering. BTreeMap keys are sorted, so we can iterate them.
        for (i, &id) in adj_map.keys().enumerate() {
            index_map.push(id);
            id_to_idx.insert(id, i);
        }

        // Build on host
        let mut adj_data = vec![T::zero(); n * n];
        let mut degrees_data = vec![T::zero(); n];

        for (i, (&_id, neighbors)) in adj_map.iter().enumerate() {
            // i matches the ordering in keys() iteration above because BTreeMap iter is ordered.

            degrees_data[i] = T::from(neighbors.len() as f32);

            for &target in neighbors {
                if let Some(&j) = id_to_idx.get(&target) {
                    adj_data[i * n + j] = T::from(1.0);
                }
            }
        }

        let matrix = B::create(&adj_data, &[n, n]);
        let degrees = B::create(&degrees_data, &[n]);

        Self {
            matrix,
            degrees,
            index_map,
        }
    }

    /// Computes the Normalized Laplacian: L = I - D^(-1/2) A D^(-1/2)
    pub fn normalized_laplacian(
        &self,
    ) -> Result<<B as TensorBackend>::Tensor<T>, CausalTensorError> {
        let n = self.index_map.len();

        let deg_vec: Vec<T> = B::to_vec(&self.degrees);

        let mut d_inv_sqrt_data = vec![T::zero(); n * n];
        for (i, &d) in deg_vec.iter().enumerate() {
            if d != T::zero() {
                let val = d.powf(T::from(-0.5));
                d_inv_sqrt_data[i * n + i] = val; // Diagonal
            }
        }

        let d_inv_sqrt = B::create(&d_inv_sqrt_data, &[n, n]);

        let mut identity_data = vec![T::zero(); n * n];
        for i in 0..n {
            identity_data[i * n + i] = T::from(1.0);
        }
        let identity = B::create(&identity_data, &[n, n]);

        let da = B::matmul(&d_inv_sqrt, &self.matrix);
        let dad = B::matmul(&da, &d_inv_sqrt);

        Ok(B::sub(&identity, &dad))
    }

    /// Simulates diffusion for k steps: X_{t+1} = A * X_t
    pub fn diffuse(
        &self,
        initial_state: &<B as TensorBackend>::Tensor<T>,
        steps: usize,
    ) -> <B as TensorBackend>::Tensor<T> {
        // Clone via add with zeros if needed, or create new.
        let shape = B::shape(initial_state);
        let zeros = B::zeros(&shape);
        let mut state = B::add(&zeros, initial_state);

        if steps == 0 {
            return state;
        }

        for _ in 0..steps {
            let next = B::matmul(&self.matrix, &state);
            state = next;
        }
        state
    }
}
